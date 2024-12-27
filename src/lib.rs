//! エントリポイントになってデシリアライズとルーティングをしている
//! src/handle.rs にルート先の関数が置かれている

mod crypt;
mod error;
mod github;
mod handle;
mod parser;
mod schedule;

use crypt::GitHubApp;
use github::GitHubEvent;
use worker::*;

use github_webhook::payload_types as gh;

use serde::de::Deserialize;

const APP_NAME: &str = "satler-bot";

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        .get_async("/", |_req, _ctx| async move {
            Response::redirect(Url::parse("https://github.com/satler-git/bot")?)
        })
        .post_async("/webhook", webhook)
        .run(req, env)
        .await
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
                        let d1 = ctx.env.d1("DB")?;

                        let installation = event.installation.as_ref().unwrap().id;
                        let token = github_app.token(installation).await?;

                        handle::issue_comment_created(event, token, d1, installation).await?;

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

#[event(scheduled)]
pub async fn scheduled_handler(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    let github_app = GitHubApp::new(
        include_str!("../secret.pem"),
        &env.secret("GITHUB_CLIENT_ID").unwrap().to_string(),
    );

    {
        let d1 = env.d1("DB").unwrap();
        schedule::auto_merge(&d1, github_app).await.unwrap();
    }
}
