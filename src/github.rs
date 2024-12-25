// The structure is inspired from https://github.com/web3infra-foundation/mega/blob/6410bc5a0a41f41d0730195f22eebaf27aa89918/taurus/src/event/github_webhook.rs
// Copyright (c) 2023 - 2024 Web3 Infrastructure Foundation
#[derive(Debug)]
pub struct GitHubEvent {
    pub _type: EventType,
    pub payload: serde_json::Value,
}

#[derive(Debug)]
pub enum EventType {
    IssueComment,
    // _Unknown(String),
    _Unknown,
}

use github_webhook::payload_types as gh;
use worker::Result;

use reqwest::header;

pub async fn comment_on_issue<'a>(
    issue: &gh::Issue<'a>,
    repo: &gh::Repository<'a>,
    content: &str,
    token: &str,
) -> Result<()> {
    let endpoint = format!(
        "https://api.github.com/repos/{}/{}/issues/{}/comments",
        repo.owner.login, repo.name, issue.number
    );

    let client = reqwest::Client::new();

    client
        .post(endpoint)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::ACCEPT, "application/vnd.github+json")
        .header(header::USER_AGENT, crate::APP_NAME)
        .body(
            serde_json::json!({
                "body": content,
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(|e| worker::Error::RustError(format!("Error in sending a request: {e}")))
        .map(|_| ())
}

impl From<&str> for EventType {
    fn from(v: &str) -> EventType {
        match v {
            "issue_comment" => Self::IssueComment,
            _ => Self::_Unknown,
        }
    }
}

impl From<String> for EventType {
    fn from(v: String) -> EventType {
        match v.as_str() {
            "issue_comment" => Self::IssueComment,
            _ => Self::_Unknown,
        }
    }
}
