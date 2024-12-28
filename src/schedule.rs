//! Scheduled handler

use crate::github::{comment_on_issue, marge_pr};
use worker::*;

pub async fn auto_merge(d1: &D1Database, github_app: crate::crypt::GitHubApp) -> Result<()> {
    #[derive(Debug, serde::Deserialize)]
    struct Res {
        id: u64,
        pr_number: u64,
        owner: String,
        repository: String,
        installation_id: u64,
    }
    console_log!("Scheduled auto merge");
    console_log!("Querying merges");
    let query = query!(
        &d1,
        "SELECT (id, pr_number, owner, repository, installation_id) FROM merge where will_merged_at < DATETIME('now') AND merged = 0"
    );

    let results = query.run().await?.results::<Res>()?;
    for ri in results {
        console_log!(
            "Merging PR: {}/{}:#{}",
            ri.owner,
            ri.repository,
            ri.pr_number
        );
        let token = github_app.token(ri.installation_id).await?;
        // // マージできるか
        // {
        //     let is_pr_mergeable =
        //         crate::github::is_pr_mergeable(ri.pr_number, &ri.owner, &ri.repository, &token)
        //             .await?;
        //
        //     if is_pr_mergeable == Some(false) {
        //         comment_on_issue(
        //             ri.pr_number,
        //             &ri.owner,
        //             &ri.repository,
        //             "Somethins were wrong. We can't merge this time",
        //             &token,
        //         )
        //         .await?;
        //         mark_as_merged(&d1, ri.id).await?; // 5分ごとにのアラームみたいになるのをさけるため
        //         return Ok(());
        //     } else if is_pr_mergeable == None {
        //         return Ok(());
        //     }
        // }
        let m = marge_pr(ri.pr_number, &ri.owner, &ri.repository, &token).await;
        if m.is_err() {
            comment_on_issue(
                ri.pr_number,
                &ri.owner,
                &ri.repository,
                "Somethins were wrong. We can't merge this time",
                &token,
            )
            .await?;
        }
        mark_as_merged(&d1, ri.id).await?;
    }

    Ok(())
}

async fn mark_as_merged(d1: &D1Database, id: u64) -> Result<()> {
    let update_query = query!(&d1, "UPDATE merge SET merged = 1 WHERE id = ?1", id)?;
    update_query.run().await?;
    Ok(())
}
