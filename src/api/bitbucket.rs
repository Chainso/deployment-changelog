use std::{fmt::Display, collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::chrono::{DateTime, Utc};
use serde_with::TimestampMilliSeconds;
use serde_with::formats::Flexible;
use anyhow::Result;

use super::api::{RestClient, Paginated};

enum BitbucketEndpoints {
    CompareCommits,
    PullRequestsForCommit,
    IssuesForPullRequest
}

impl BitbucketEndpoints {
    fn url(&self) -> &'static str {
        match self {
            BitbucketEndpoints::CompareCommits => "rest/api/latest/projects/{projectKey}/repos/{repositorySlug}/compare/commits?from={from}&to={to}",
            BitbucketEndpoints::PullRequestsForCommit => "rest/api/latest/projects/{projectKey}/repos/{repositorySlug}/commits/{commitId}/pull-requests",
            BitbucketEndpoints::IssuesForPullRequest => "/rest/jira/latest/projects/{projectKey}/repos/{repositorySlug}/pull-requests/{pullRequestId}/issues"
        }
    }
}

enum BitbucketOptions {
    PageStart
}

impl BitbucketOptions {
    fn option(&self) -> &'static str {
        match self {
            BitbucketOptions::PageStart => "start"
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
    pub limit: u32,
    pub next_page_start: Option<u32>
}

impl<T: Serialize> Display for BitbucketPage<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket commit page: {error}")
        }
    }
}

pub struct BitbucketPaginated<'a, T> {
    client: &'a BitbucketClient,
    url: String,
    query: HashMap<String, String>,
    next_page_start: Option<u32>,
    is_last_page: bool,
    phantom: PhantomData<T>
}

impl<'a, T> BitbucketPaginated<'a, T> {
    fn new(client: &'a BitbucketClient, url: String, query: Option<&HashMap<String, String>>) -> Self {
        let query_options = match query {
            Some(query_opts) => query_opts.clone(),
            None => HashMap::with_capacity(1)
        };

        BitbucketPaginated {
            client,
            url,
            query: query_options,
            next_page_start: Some(0),
            is_last_page: false,
            phantom: PhantomData
        }
    }
}

#[async_trait::async_trait]
impl<T: DeserializeOwned + Send> Paginated<T> for BitbucketPaginated<'_, T> {
    async fn next(&mut self) -> Result<Vec<T>> {
        if let Some(next_page_start) = self.next_page_start {
            self.query.insert(
                BitbucketOptions::PageStart.option().to_string(),
                next_page_start.to_string()
            );
        };

        let page = self.client.client.get::<BitbucketPage<T>>(&self.url, Some(&self.query)).await?;

        self.next_page_start = page.next_page_start;
        self.is_last_page = page.is_last_page;

        Ok(page.values)
    }

    fn is_last(&self) -> bool {
        self.is_last_page
    }
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
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

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketPullRequest {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub open: bool,
    pub author: BitbucketPullRequestAuthor,

    #[serde_as(as = "TimestampMilliSeconds<String, Flexible>")]
    pub created_date: DateTime<Utc>,

    #[serde_as(as = "TimestampMilliSeconds<String, Flexible>")]
    pub updated_date: DateTime<Utc>
}

impl Display for BitbucketPullRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket pull request: {error}")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug)]
pub struct BitbucketClient {
    client: RestClient
}

impl BitbucketClient {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: RestClient::new(base_url)?
        })
    }

    pub fn from_client(client: RestClient) -> Self {
        Self {
            client
        }
    }

    pub fn compare_commits(&self, project: &str, repo: &str, start_commit: &str, end_commit: &str) -> BitbucketPaginated<BitbucketCommit> {
        let compare_commits_path: String = BitbucketEndpoints::CompareCommits.url()
            .replace("{projectKey}", project)
            .replace("{repositorySlug}", repo)
            .replace("{from}", start_commit)
            .replace("{to}", end_commit);

        BitbucketPaginated::new(&self, compare_commits_path, None)
    }

    pub fn get_pull_requests(&self, project: &str, repo: &str, commit: &str) -> BitbucketPaginated<BitbucketPullRequest> {
        let get_pull_requests_path: String = BitbucketEndpoints::PullRequestsForCommit.url()
            .replace("{projectKey}", project)
            .replace("{repositorySlug}", repo)
            .replace("{commitId}", commit);

        BitbucketPaginated::new(&self, get_pull_requests_path, None)
    }

    pub async fn get_pull_request_issues(&self, project: &str, repo: &str, pull_request_id: u64) -> Result<Vec<BitbucketPullRequestIssue>> {
        let get_pull_request_issues_path: String = BitbucketEndpoints::IssuesForPullRequest.url()
            .replace("{projectKey}", project)
            .replace("{repositorySlug}", repo)
            .replace("{pullRequestId}", &pull_request_id.to_string());

        self.client.get::<Vec<BitbucketPullRequestIssue>>(&get_pull_request_issues_path, None).await
    }
}
