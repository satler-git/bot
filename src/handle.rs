use github_webhook::payload_types as gh;
use worker::*;

pub async fn issue_comment_created<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: String,
) -> Result<()> {
    Ok(())
}
