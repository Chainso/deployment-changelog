use crate::api::{jira::{JiraIssue, JiraClient}, bitbucket::{BitbucketCommit, BitbucketPullRequest, BitbucketPullRequestIssue, BitbucketClient}};
use std::fmt::Display;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Changelog {
    pub commits: Vec<BitbucketCommit>,
    pub pull_requests: Vec<BitbucketPullRequest>,
    pub issues: Vec<JiraIssue>
}

impl Display for Changelog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing changelog: {error}")
        }
    }
}

pub async fn get_changelog(
    bitbucket_client: &BitbucketClient,
    jira_client: &JiraClient,
    project: &str,
    repo: &str,
    start_commit: &str,
    end_commit: &str
) -> Changelog {
    let commits: Vec<BitbucketCommit> = bitbucket_client.compare_commits(
        project,
        repo,
        start_commit,
        end_commit
    )
        .await
        .values;

    let pull_requests: Vec<BitbucketPullRequest> = futures::future::join_all(
        commits.iter()
            .map(|commit| bitbucket_client.get_pull_requests(project, repo, &commit.id))
    )
        .await
        .into_iter()
        .flat_map(|pull_request_page| pull_request_page.values)
        .collect();

    let pull_request_issues: Vec<BitbucketPullRequestIssue> = futures::future::join_all(
        pull_requests.iter()
            .map(|pull_request| bitbucket_client.get_pull_request_issues(project, repo, pull_request.id))
    )
        .await
        .into_iter()
        .flatten()
        .collect();

    let issues = futures::future::join_all(
        pull_request_issues.iter()
            .map(|pull_request_issue| jira_client.get_issue(&pull_request_issue.key))
    ).await;

    Changelog {
        commits,
        pull_requests,
        issues
    }
}
