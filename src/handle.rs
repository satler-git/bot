use chrono::{FixedOffset, NaiveDateTime, Utc};
use github_webhook::payload_types as gh;
use worker::*;

use crate::parser::{Command, Help, Merge};

use crate::github::comment_on_issue;

const MENTION: &str = "@satler-bot";

pub async fn issue_comment_created<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: String,
    d1: D1Database,
) -> Result<()> {
    let input = event.comment.body;
    let command = crate::parser::Command::try_parse(input, MENTION);

    // worker::console_debug!("{command:?}");

    let issue = &event.issue.issue;
    let repo = &event.repository;

    if "satler-git" // TODO:人を変更できるようにする？
        != event.comment.user.login // TODO: 送った人にメンション?
        || command.is_err()
    {
        if let Err(crate::parser::error::Error::NotACommand) = command {
            // メンションされたけど正しくない場合
            comment_on_issue(
                issue,
                repo,
                "Some syntax is wrong. View the help with the`help` command",
                &token,
            )
            .await?;
        }
        if Some("satler-git") != event.comment.user.name && command.is_ok() {
            worker::console_debug!("{:?}", event.comment.user.name);

            comment_on_issue(
                issue,
                repo,
                "You are not authorised to operate this operation here",
                &token,
            )
            .await?;
        }
        return Ok(());
    }

    let command = command.unwrap();

    match command {
        Command::Help => comment_on_issue(issue, repo, Command::HELP, &token).await?,
        Command::Merge(merge) => match merge {
            Merge::Add(date) => handle_merge_add(event, &token, date, d1).await?,
            Merge::Cancel => handle_merge_cancel(event, &token, d1).await?,
            Merge::Help => comment_on_issue(issue, repo, Merge::HELP, &token).await?,
        },
    }
    Ok(())
}

async fn handle_merge_add<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: &str,
    date: NaiveDateTime,
    d1: D1Database,
) -> Result<()> {
    let issue = &event.issue.issue;
    let repo = &event.repository;
    // Issueな場合
    {
        if issue.pull_request.is_none() {
            comment_on_issue(
                issue,
                repo,
                "This operation can only be performed on Pull Requests.",
                token,
            )
            .await?;
            return Ok(());
        }
    }
    // 既にマージされている場合
    let pr = issue.pull_request.as_ref().unwrap();
    {
        if pr.merged_at.is_some() {
            comment_on_issue(
                issue,
                repo,
                "It is not possible to run this command on the merged Pull Request",
                token,
            )
            .await?;
            return Ok(());
        }
    }
    // 過ぎている場合
    let tz = FixedOffset::east_opt(9 * 3600).unwrap();
    {
        let now = Utc::now().with_timezone(&tz).naive_local();

        if now > date {
            comment_on_issue(
                issue,
                repo,
                "It is not possible to specify a time past",
                token,
            )
            .await?;
            return Ok(());
        }
    }

    let date_utc = date - std::time::Duration::from_secs(9 * 3600);
    let insert_merge_query = worker::query!(
        &d1,
        "INSERT INTO merge (pr_number, owner, repository, will_merged_at) VALUES (?1, ?2, ?3, ?4)",
        &issue.number,
        &repo.owner.login,
        &repo.name,
        &date_utc.to_string(),
    )?;

    let result = d1.batch(vec![insert_merge_query]).await?;

    if result[0].success() == false {
        return Err(worker::Error::RustError(format!(
            "{}",
            result[0].error().unwrap()
        )));
    }

    comment_on_issue(
        issue,
        repo,
        "Automatic merging has been successfully scheduled.",
        token,
    )
    .await?;

    Ok(())
}

async fn handle_merge_cancel<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: &str,
    d1: D1Database,
) -> Result<()> {
    Ok(()) // TODO:
}
