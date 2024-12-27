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
            Merge::Add(date) => handle_merge_add(event, &token, date, &d1).await?,
            Merge::Cancel => handle_merge_cancel(event, &token, &d1).await?,
            Merge::Help => comment_on_issue(issue, repo, Merge::HELP, &token).await?,
        },
    }
    Ok(())
}

async fn handle_merge_add<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: &str,
    date: NaiveDateTime,
    d1: &D1Database,
) -> Result<()> {
    let issue = &event.issue.issue;
    let repo = &event.repository;
    // Issueな場合
    {
        if issue.pull_request.is_none() {
            comment_on_issue(
                issue,
                repo,
                "This operation can only be performed on Pull Requests",
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

    // すでにスケジュール済み
    let owner = &repo.owner.login;
    let repo_name = &repo.name;
    let issue_num = issue.number;
    {
        if is_already_merged(owner, repo_name, issue_num, d1)
            .await?
            .is_some()
        {
            comment_on_issue(
                issue,
                repo,
                "It is not possible to schedule in a merged Pull Request",
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
        &issue_num,
        &owner,
        &repo_name,
        &date_utc.to_string(),
    )?;

    let result = d1.batch(vec![insert_merge_query]).await?;

    if !result[0].success() {
        return Err(worker::Error::RustError(
            result[0].error().unwrap().to_string(),
        ));
    }

    comment_on_issue(
        issue,
        repo,
        "Automatic merging has been successfully scheduled",
        token,
    )
    .await?;

    Ok(())
}

async fn handle_merge_cancel<'a>(
    event: gh::IssueCommentCreatedEvent<'a>,
    token: &str,
    d1: &D1Database,
) -> Result<()> {
    let issue = &event.issue.issue; // TODO: 上と共通だから切りだす
    let repo = &event.repository;
    // Issueな場合
    {
        if issue.pull_request.is_none() {
            comment_on_issue(
                issue,
                repo,
                "This operation can only be performed on Pull Requests",
                token,
            )
            .await?;
            return Ok(());
        }
    }

    let owner = &repo.owner.login;
    let repo_name = &repo.name;
    let issue_num = issue.number;

    {
        let merged = is_already_merged(owner, repo_name, issue_num, d1).await?;
        // select してマージされていたらコメント
        if merged.is_none() {
            comment_on_issue(
                issue,
                repo,
                "It is not possible to cancel in a Pull Request that does not have an automatic merge scheduled",
                token,
            )
            .await?;
            return Ok(());
        // スケジュールされていなかったらコメント
        } else if merged == Some(true) {
            comment_on_issue(
                issue,
                repo,
                "It is not possible to cancel in a pull request that has been automatically merged",
                token,
            )
            .await?;
            return Ok(());
        }
    }

    // 多分エラー処理おわったから削除
    let delete_merge_query = worker::query!(
        &d1,
        "DELETE FROM merge WHERE (pr_number, owner, repository) = (?1, ?2, ?3)",
        &issue_num,
        &owner,
        &repo_name,
    )?;

    let result = d1.batch(vec![delete_merge_query]).await?;

    if !result[0].success() {
        return Err(worker::Error::RustError(
            result[0].error().unwrap().to_string(),
        ));
    }

    comment_on_issue(
        issue,
        repo,
        "The automatic merge has been successfully cancelled.",
        token,
    )
    .await?;

    Ok(())
}

/// -> (マージ済み)
async fn is_already_merged(
    owner: &str,
    repo: &str,
    number: u64,
    d1: &D1Database,
) -> Result<Option<bool>> {
    #[derive(Debug, serde::Deserialize)]
    struct Res {
        merged: u64,
    }

    let query = query!(
        &d1,
        "SELECT merged FROM merge WHERE (pr_number, owner, repository) = (?1, ?2, ?3)",
        number,
        owner,
        &repo,
    )?;

    let result = query.run().await?.results::<Res>()?;

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result[0].merged == 1))
    }
}
