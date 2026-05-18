use std::path::Path;

use serde_json::Value;

use crate::launch_stack::LaunchStackReviewThread;
use crate::launch_stack_github_support::{public_text, run_gh_json};

pub(crate) fn fetch_unresolved_review_threads(
    workspace: &Path,
    pr_id: &str,
) -> Result<Vec<LaunchStackReviewThread>, String> {
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
                  isOutdated
                  path
                  line
                  comments(first: 1) {
                    nodes {
                      url
                      author {
                        login
                      }
                    }
                  }
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
        .filter_map(review_thread_from_node)
        .collect())
}

fn review_thread_from_node(node: &Value) -> Option<LaunchStackReviewThread> {
    let first_comment = node
        .pointer("/comments/nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.first());
    let url = first_comment
        .and_then(|comment| comment.get("url"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let path = node.get("path").and_then(Value::as_str).unwrap_or("");
    if url.trim().is_empty() && path.trim().is_empty() {
        return None;
    }
    Some(LaunchStackReviewThread {
        url: public_text(url, 240),
        path: public_text(path, 180),
        line: node.get("line").and_then(Value::as_u64),
        author: first_comment
            .and_then(|comment| comment.pointer("/author/login"))
            .and_then(Value::as_str)
            .map(|author| public_text(author, 80)),
        outdated: node
            .get("isOutdated")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::review_thread_from_node;

    #[test]
    fn review_thread_from_node_drops_unactionable_empty_threads() {
        assert!(review_thread_from_node(&json!({"comments": {"nodes": []}})).is_none());

        let detail = review_thread_from_node(&json!({
            "path": "src/lib.rs",
            "line": 42,
            "isOutdated": false,
            "comments": {
                "nodes": [
                    {
                        "url": "https://github.com/example/repo/pull/1#discussion_r1",
                        "author": {"login": "copilot-pull-request-reviewer"}
                    }
                ]
            }
        }))
        .expect("actionable review thread detail");

        assert_eq!(detail.path, "src/lib.rs");
        assert_eq!(detail.line, Some(42));
        assert_eq!(
            detail.url,
            "https://github.com/example/repo/pull/1#discussion_r1"
        );
    }
}
