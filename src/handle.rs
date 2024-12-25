use github_webhook::payload_types as gh;
use worker::*;

const MENTION: &str = "@satler-bot";

pub async fn issue_comment_created<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: String,
) -> Result<()> {
    let input = event.comment.body;
    let command = bot_parser::Command::try_parse(input, MENTION);

    // worker::console_debug!("{command:?}");

    if Some("satler-git") // TODO:
        != event.comment.user.name
        || command.is_err()
    {
        if Some("satler-git") != event.comment.user.name && command.is_ok() {
            // TODO: You have not enough permission to run this here
        }
        return Ok(());
    }
    Ok(())
}
