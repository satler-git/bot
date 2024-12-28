// The structure is inspired from https://github.com/web3infra-foundation/mega/blob/6410bc5a0a41f41d0730195f22eebaf27aa89918/taurus/src/event/github_webhook.rs
// Copyright (c) 2023 - 2024 Web3 Infrastructure Foundation
use serde::de::Deserialize;
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
use worker::{console_debug, Result};

use reqwest::header;

// TODO: pr_number: u64, owner: &str, repo: &str,をつくる

pub async fn comment_on_issue<'a>(
    number: u64,
    owner: &str,
    repo: &str,
    content: &str,
    token: &str,
) -> Result<()> {
    let endpoint = format!(
        "https://api.github.com/repos/{}/{}/issues/{}/comments",
        owner, repo, number
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

pub async fn is_pr_mergeable(
    pr_number: u64,
    owner: &str,
    repo: &str,
    token: &str,
) -> Result<Option<bool>> {
    let endpoint = format!(
        "https://api.github.com/repos/{}/{}/issues/{}/comments",
        owner, repo, pr_number
    );

    let client = reqwest::Client::new();

    let res = client
        .get(endpoint)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::ACCEPT, "application/vnd.github+json")
        .header(header::USER_AGENT, crate::APP_NAME)
        .send()
        .await
        .map_err(|e| worker::Error::RustError(format!("Error in sending a request: {e}")))?
        .text()
        .await
        .map_err(|e| {
            worker::Error::RustError(format!("Error in reading text from the body: {e}"))
        })?;

    let payload: serde_json::Value =
        serde_json::from_str(&res).map_err(worker::Error::SerdeJsonError)?;

    let pr = gh::PullRequest::deserialize(&payload).map_err(worker::Error::SerdeJsonError)?;

    Ok(pr.mergeable)
}

pub async fn marge_pr(pr_number: u64, owner: &str, repo: &str, token: &str) -> Result<()> {
    let endpoint = format!("https://api.github.com/repos/{owner}/{repo}/pulls/{pr_number}/merge");
    #[derive(Debug, serde::Deserialize)]
    struct Res {
        // sha: String,
        merged: bool,
        message: String,
    }

    let client = reqwest::Client::new();

    let res = client
        .put(endpoint)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::ACCEPT, "application/vnd.github+json")
        .header(header::USER_AGENT, crate::APP_NAME)
        .send()
        .await
        .map_err(|e| worker::Error::RustError(format!("Error in sending a request: {e}")))?
        .text()
        .await
        .map_err(|e| {
            worker::Error::RustError(format!("Error in reading text from the body: {e}"))
        })?;

    console_debug!("/merge response: {res:?}");

    let res: Res = serde_json::from_str(&res).map_err(worker::Error::SerdeJsonError)?;
    if !res.merged {
        Err(worker::Error::RustError(format!(
            "Pr havn't been merged: {}",
            res.message
        )))
    } else {
        Ok(())
    }
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
