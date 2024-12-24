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
