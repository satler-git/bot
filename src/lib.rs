mod crypt;
mod error;
mod github;
mod handle;

use crypt::GitHubApp;
use github::GitHubEvent;
use worker::*;

use github_webhook::payload_types as gh;

use serde::de::Deserialize;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router.post_async("/webhook", webhook).run(req, env).await
}

async fn webhook(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let github_app = GitHubApp::new(
        include_str!("../secret.pem"),
        &ctx.secret("GITHUB_CLIENT_ID")?.to_string(),
    );

    let webhook_sec: String = ctx.secret("WEBHOOK_SEC")?.to_string();
    let signature: Option<String> = req.headers().get("X-Hub-Signature-256")?;

    let mut req = req;

    if let Some(sig) = signature {
        let body = req.text().await?;

        if !crypt::verify_signature(&body, &webhook_sec, &sig[7..] /* sha256= */) {
            return Response::error("Unauthorised (signature did not match)", 401);
        }

        let github_event = {
            let event_type = req.headers().get("X-GitHub-Event")?.unwrap().into();
            GitHubEvent {
                _type: event_type,
                payload: serde_json::from_str(&body).map_err(Error::SerdeJsonError)?,
            }
        };

        match github_event._type {
            github::EventType::IssueComment => {
                let issue_comment_event = gh::IssueCommentEvent::deserialize(&github_event.payload)
                    .map_err(Error::SerdeJsonError)?;

                match issue_comment_event {
                    gh::IssueCommentEvent::Created(event) => {
                        let token = github_app
                            .token(event.installation.as_ref().unwrap().id)
                            .await?;

                        handle::issue_comment_created(event, token).await?;

                        Response::empty()
                    }
                    gh::IssueCommentEvent::Edited(_) => Response::empty(),
                    gh::IssueCommentEvent::Deleted(_) => Response::empty(),
                }
            }
            _ => Response::empty(),
        }
    } else {
        Response::error("Unauthorised (signature does not exit)", 401)
    }
}
