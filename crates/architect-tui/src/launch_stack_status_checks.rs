use std::path::Path;

use serde_json::{Value, json};

use crate::launch_stack_github_support::run_gh_json;

pub(crate) fn fetch_status_check_rollup(workspace: &Path, pr_id: &str) -> Result<Value, String> {
    let query = r#"
        query($id: ID!) {
          node(id: $id) {
            ... on PullRequest {
              commits(last: 1) {
                nodes {
                  commit {
                    statusCheckRollup {
                      contexts(first: 100) {
                        pageInfo {
                          hasNextPage
                        }
                        nodes {
                          __typename
                          ... on CheckRun {
                            name
                            status
                            conclusion
                            startedAt
                            completedAt
                            detailsUrl
                            checkSuite {
                              app {
                                databaseId
                                id
                                slug
                                name
                              }
                            }
                          }
                          ... on StatusContext {
                            context
                            state
                            createdAt
                            targetUrl
                          }
                        }
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
    status_check_rollup_from_graphql_value(&value)
}

fn status_check_rollup_from_graphql_value(value: &Value) -> Result<Value, String> {
    let commit = value
        .pointer("/data/node/commits/nodes/0/commit")
        .ok_or_else(|| "GitHub status-check evidence was unavailable".to_string())?;
    let Some(contexts) = commit.pointer("/statusCheckRollup/contexts") else {
        return Ok(json!([]));
    };
    if contexts
        .pointer("/pageInfo/hasNextPage")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(
            "GitHub status-check evidence exceeded one page; inspect checks manually".to_string(),
        );
    }
    let Some(nodes) = contexts.get("nodes").and_then(Value::as_array) else {
        return Err("GitHub status-check evidence was unavailable".to_string());
    };
    Ok(Value::Array(nodes.clone()))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::status_check_rollup_from_graphql_value;

    #[test]
    fn status_check_rollup_from_graphql_value_keeps_app_identity_payload() {
        let rollup = status_check_rollup_from_graphql_value(&json!({
            "data": {
                "node": {
                    "commits": {
                        "nodes": [{
                            "commit": {
                                "statusCheckRollup": {
                                    "contexts": {
                                        "pageInfo": {"hasNextPage": false},
                                        "nodes": [{
                                            "__typename": "CheckRun",
                                            "name": "verify",
                                            "status": "COMPLETED",
                                            "conclusion": "SUCCESS",
                                            "checkSuite": {
                                                "app": {
                                                    "databaseId": 15368,
                                                    "slug": "github-actions"
                                                }
                                            }
                                        }]
                                    }
                                }
                            }
                        }]
                    }
                }
            }
        }))
        .expect("rollup payload");

        assert_eq!(
            rollup.pointer("/0/checkSuite/app/slug"),
            Some(&json!("github-actions"))
        );
    }

    #[test]
    fn status_check_rollup_from_graphql_value_fails_closed_on_pagination() {
        let error = status_check_rollup_from_graphql_value(&json!({
            "data": {
                "node": {
                    "commits": {
                        "nodes": [{
                            "commit": {
                                "statusCheckRollup": {
                                    "contexts": {
                                        "pageInfo": {"hasNextPage": true},
                                        "nodes": []
                                    }
                                }
                            }
                        }]
                    }
                }
            }
        }))
        .expect_err("paginated rollup should fail closed");

        assert!(error.contains("exceeded one page"));
    }
}
