use chrono::{FixedOffset, NaiveDateTime, Utc};
use github_webhook::payload_types as gh;
use worker::*;

use bot_parser::{Command, Help, Merge};

const MENTION: &str = "@satler-bot";

pub async fn issue_comment_created<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: String,
) -> Result<()> {
    let input = event.comment.body;
    let command = bot_parser::Command::try_parse(input, MENTION);

    // worker::console_debug!("{command:?}");

    let issue = &event.issue.issue;

    if "satler-git" // TODO:人を変更できるようにする？
        != event.comment.user.login
        || command.is_err()
    {
        if let Err(bot_parser::error::Error::NotACommand(_)) = command {
            // メンションされたけど正しくない場合
            comment_on_issue(
                issue,
                "Some syntax is wrong. View the help with the`help` command",
                &token,
            )
            .await?;
        }
        if Some("satler-git") != event.comment.user.name && command.is_ok() {
            worker::console_debug!("{:?}", event.comment.user.name);

            comment_on_issue(
                issue,
                "You are not authorised to operate this operation here",
                &token,
            )
            .await?;
        }
        return Ok(());
    }

    let command = command.unwrap();

    match command {
        Command::Help => comment_on_issue(issue, Command::HELP, &token).await?,
        Command::Merge(merge) => match merge {
            Merge::Add(date) => handle_merge_add(event, &token, date).await?,
            Merge::Cancel => handle_merge_cancel(event, &token).await?,
            Merge::Help => comment_on_issue(issue, Merge::HELP, &token).await?,
        },
    }
    Ok(())
}

use std::cell::LazyCell;

const ASIA_TOKYO: LazyCell<FixedOffset> =
    LazyCell::new(|| FixedOffset::east_opt(5 * 3600).unwrap());

async fn handle_merge_add<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: &str,
    date: NaiveDateTime,
) -> Result<()> {
    // Issueな場合
    {
        let issue = &event.issue.issue;
        if issue.pull_request.is_none() {
            comment_on_issue(
                issue,
                "This operation can only be performed on Pull Requests.",
                token,
            )
            .await?;
            return Ok(());
        }
    }
    // 過ぎている場合
    {
        let issue = &event.issue.issue;
        let now = Utc::now().with_timezone(&*ASIA_TOKYO).naive_local();
        if now > date {
            comment_on_issue(issue, "It is not possible to specify a time past", token).await?;
            return Ok(());
        }
    }
    // 既にマージされている場合
    {
        let issue = &event.issue.issue;
        let pr = issue.pull_request.as_ref().unwrap();
        if pr.merged_at.is_some() {
            comment_on_issue(
                issue,
                "It is not possible to run this command on the merged Pull Request",
                token,
            )
            .await?;
            return Ok(());
        }
    }

    Ok(()) // TODO:
}

async fn handle_merge_cancel<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: &str,
) -> Result<()> {
    Ok(()) // TODO:
}

async fn comment_on_issue<'a>(issue: &gh::Issue<'a>, content: &str, token: &str) -> Result<()> {
    worker::console_debug!("{content}"); // TODO:
    Ok(())
}
