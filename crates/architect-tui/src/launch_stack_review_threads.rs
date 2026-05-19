use std::path::Path;

use serde_json::Value;

use crate::launch_stack_github_support::run_gh_json;

pub(crate) fn fetch_unresolved_review_threads(
    workspace: &Path,
    pr_id: &str,
) -> Result<usize, String> {
    let query = r#"
        query($id: ID!) {
          node(id: $id) {
            ... on PullRequest {
              reviewThreads(first: 100) {
                pageInfo {
                  hasNextPage
                }
                nodes {
                  isResolved
                }
              }
            }
          }
        }
    "#;
    let args = vec![
        "api".to_string(),
        "graphql".to_string(),
        "-f".to_string(),
        format!("query={query}"),
        "-f".to_string(),
        format!("id={pr_id}"),
    ];
    let value = run_gh_json(workspace, &args)?;
    let threads = value
        .pointer("/data/node/reviewThreads")
        .ok_or_else(|| "GitHub review-thread evidence was unavailable".to_string())?;
    if threads
        .pointer("/pageInfo/hasNextPage")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(
            "GitHub review-thread evidence exceeded one page; inspect unresolved threads manually"
                .to_string(),
        );
    }
    let Some(nodes) = threads.get("nodes").and_then(Value::as_array) else {
        return Err("GitHub review-thread evidence was unavailable".to_string());
    };
    Ok(nodes
        .iter()
        .filter(|node| {
            !node
                .get("isResolved")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count())
}
