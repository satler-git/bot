mod crypt;
mod error;
mod github;

use github::GitHubEvent;
use worker::*;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router.post_async("/webhook", webhook).run(req, env).await
}

async fn webhook(req: Request, ctx: RouteContext<()>) -> Result<Response> {
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
                payload: body.into(),
            }
        };

        match github_event._type {
            github::EventType::IssueComment => {
                let issue_comment_event: ocho_gato::IssueCommentEvent =
                    serde_json::from_value(github_event.payload)
                        .map_err(worker::Error::SerdeJsonError)?;

                match issue_comment_event {
                    ocho_gato::IssueCommentEvent::Created(event) => {
                        todo!()
                    }
                    ocho_gato::IssueCommentEvent::Deleted(_) => Response::empty(),
                    ocho_gato::IssueCommentEvent::Edited(_) => Response::empty(),
                }
            }
            _ => Response::empty(),
        }
    } else {
        Response::error("Unauthorised (signature does not exit)", 401)
    }
}
