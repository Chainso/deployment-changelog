use std::fmt::Display;

use serde::{Deserialize, Serialize};
use anyhow::Result;

use super::api::RestClient;

enum BitbucketEndpoints {
    CompareCommits,
    PullRequestsForCommit,
    IssuesForPullRequest
}

impl BitbucketEndpoints {
    fn url(&self) -> &str {
        match self {
            BitbucketEndpoints::CompareCommits => "rest/api/1.0/projects/{projectKey}/repos/{repositorySlug}/compare/commits?from={from}&to={to}",
            BitbucketEndpoints::PullRequestsForCommit => "rest/api/1.0/projects/{projectKey}/repos/{repositorySlug}/commits/{commitId}/pull-requests",
            BitbucketEndpoints::IssuesForPullRequest => "/rest/jira/1.0/projects/{projectKey}/repos/{repositorySlug}/pull-requests/{pullRequestId}/issues"
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketPage<T> {
    pub values: Vec<T>,
    pub size: u32,
    pub is_last_page: bool,
    pub start: u32,
    pub limit: u32
}

impl<T: Serialize> Display for BitbucketPage<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket commit page: {error}")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketCommit {
    pub id: String,
    pub display_id: String,
    pub author: BitbucketAuthor,
    pub committer: BitbucketAuthor,
    pub message: String
}

impl Display for BitbucketCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket commit: {error}")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketAuthor {
    pub name: String,
    pub email_address: String,
    pub display_name: String
}

impl Display for BitbucketAuthor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket author: {error}")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketPullRequest {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub open: bool,
    pub author: BitbucketPullRequestAuthor
}

impl Display for BitbucketPullRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket pull request: {error}")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketPullRequestAuthor {
    pub user: BitbucketAuthor,
    pub approved: bool
}

impl Display for BitbucketPullRequestAuthor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket pull request author: {error}")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketPullRequestIssue {
    pub key: String,
    pub url: String
}

impl Display for BitbucketPullRequestIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket pull request issue: {error}")
        }
    }
}

pub struct BitbucketClient {
    client: RestClient
}

impl BitbucketClient {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: RestClient::new(base_url)?
        })
    }

    pub async fn compare_commits(&self, project: &str, repo: &str, start_commit: &str, end_commit: &str) -> BitbucketPage<BitbucketCommit> {
        let compare_commits_path: &str = &BitbucketEndpoints::CompareCommits.url()
            .replace("{projectKey}", project)
            .replace("{repositorySlug}", repo)
            .replace("{from}", start_commit)
            .replace("{to}", end_commit);

        self.client.get::<BitbucketPage<BitbucketCommit>>(&compare_commits_path, None).await
    }

    pub async fn get_pull_requests(&self, project: &str, repo: &str, commit: &str) -> BitbucketPage<BitbucketPullRequest> {
        let get_pull_requests_path: &str = &BitbucketEndpoints::PullRequestsForCommit.url()
            .replace("{projectKey}", project)
            .replace("{repositorySlug}", repo)
            .replace("{commitId}", commit);

        self.client.get::<BitbucketPage<BitbucketPullRequest>>(&get_pull_requests_path, None).await
    }

    pub async fn get_pull_request_issues(&self, project: &str, repo: &str, pull_request_id: u64) -> Vec<BitbucketPullRequestIssue> {
        let get_pull_requests_path: &str = &BitbucketEndpoints::IssuesForPullRequest.url()
            .replace("{projectKey}", project)
            .replace("{repositorySlug}", repo)
            .replace("{pullRequestId}", &pull_request_id.to_string());

        self.client.get::<Vec<BitbucketPullRequestIssue>>(&get_pull_requests_path, None).await
    }
}
