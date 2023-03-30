//! The `deployment_changelog::api::bitbucket` module provides a high-level API for interacting with
//! the Bitbucket REST API, making it easy to retrieve information related to commits, pull requests,
//! and issues.
//!
//! This module provides the `BitbucketClient` struct for interacting with the Bitbucket API, as well
//! as various structs for representing Bitbucket objects, such as `BitbucketCommit`, `BitbucketPullRequest`,
//! and `BitbucketPullRequestIssue`.
//!
//! # Examples
//!
//! Creating a new `BitbucketClient` with a base URL and fetching commits between two revisions:
//!
//! ```rust
//! use deployment_changelog::api::bitbucket::BitbucketClient;
//!
//! let bitbucket_client = BitbucketClient::new("https://api.bitbucket.org")
//!     .unwrap();
//!
//! let mut commits = bitbucket_client.compare_commits("MY_PROJECT", "MY_REPO", "abcdef123456", "fedcba654321");
//!
//! let all_commits = commits.all().await.unwrap();
//!
//! for commit in all_commits {
//!     println!("{}", commit);
//! }
//! ```
//!
//! Fetching pull requests for a specific commit:
//!
//! ```rust
//! use deployment_changelog::api::bitbucket::BitbucketClient;
//!
//! let bitbucket_client = BitbucketClient::new("https://api.bitbucket.org")
//!     .unwrap();
//!
//! let mut pull_requests = bitbucket_client.get_pull_requests("MY_PROJECT", "MY_REPO", "abcdef123456");
//!
//! let all_pull_requests = pull_requests.all().await.unwrap();
//!
//! for pr in all_pull_requests {
//!     println!("{}", pr);
//! }
//! ```
//!
//! Fetching issues associated with a pull request:
//!
//! ```rust
//! use deployment_changelog::api::bitbucket::BitbucketClient;
//!
//! let bitbucket_client = BitbucketClient::new("https://api.bitbucket.org")
//!     .unwrap();
//!
//! let issues = bitbucket_client.get_pull_request_issues("MY_PROJECT", "MY_REPO", 42).await.unwrap();
//!
//! for issue in issues {
//!     println!("{}", issue);
//! }
//! ```
use std::{fmt::Display, collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::chrono::{DateTime, Local};
use serde_with::TimestampMilliSeconds;
use serde_with::formats::Flexible;
use anyhow::Result;

use super::rest::{RestClient, Paginated};

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

/// The `BitbucketPage` struct represents a single page of results returned by the Bitbucket API.
///
/// It is generic over the type `T` and contains a vector of values, pagination information such as the
/// current page size, whether this is the last page, the current page start index, the result limit,
/// and the index for the next page, if available.
///
/// You usually don't need to interact with `BitbucketPage` directly, as the `BitbucketPaginated`
/// iterator takes care of the pagination for you when fetching multiple pages of results.
///
/// # Example
///
/// Suppose you are fetching commits using the `BitbucketClient::compare_commits()` method. The
/// response from the Bitbucket API will be represented as a `BitbucketPage<BitbucketCommit>`.
///
/// To get the vector of `BitbucketCommit` objects from the page, you can access the `values` field:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::{BitbucketClient, BitbucketPage};
///
/// // Suppose you fetched a BitbucketPage<BitbucketCommit> named 'commit_page'
/// let commits: Vec<BitbucketCommit> = commit_page.values;
///
/// for commit in commits {
///     println!("{}", commit);
/// }
/// ```
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

/// The `BitbucketPaginated` struct represents an iterator for paginated results returned by the
/// Bitbucket API.
///
/// It is generic over the type `T`, and is used in conjunction with [`Paginated`](create::api::Paginated) trait.
/// It abstracts the pagination logic, allowing you to easily fetch multiple pages of results without
/// worrying about pagination details.
///
/// You usually don't need to create a `BitbucketPaginated` object manually, as the methods from `BitbucketClient`
/// will return a `BitbucketPaginated` instance when necessary.
///
/// # Example
///
/// Suppose you want to fetch all commits between two commit hashes using the `BitbucketClient::compare_commits()` method.
/// It returns a `BitbucketPaginated<BitbucketCommit>` iterator, which you can use to fetch all pages of results:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::{BitbucketClient, BitbucketPaginated};
/// use deployment_changelog::api::rest::Paginated;
///
/// // Suppose you have a BitbucketClient named 'client'
/// let project_key = "PROJECT";
/// let repo_slug = "my-repo";
/// let start_commit = "abcdef";
/// let end_commit = "123456";
///
/// let mut commits_iter = client.compare_commits(project_key, repo_slug, start_commit, end_commit);
/// let all_commits = commits_iter.all().await.unwrap();
///
/// for commit in all_commits {
///     println!("{}", commit);
/// }
/// ```
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

/// The `BitbucketCommit` struct represents a single commit returned by the Bitbucket API.
///
/// It contains information about the commit, such as its ID, display ID, author, committer, and message.
///
/// This struct is usually used as a result of API calls made through the `BitbucketClient`.
///
/// # Example
///
/// Suppose you want to fetch all commits between two commit hashes using the `BitbucketClient::compare_commits()` method.
/// You'll receive a `BitbucketPaginated<BitbucketCommit>` iterator, which you can use to fetch all pages of commits:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::{BitbucketClient, BitbucketPaginated};
/// use deployment_changelog::api::rest::Paginated;
///
/// // Suppose you have a BitbucketClient named 'client'
/// let project_key = "PROJECT";
/// let repo_slug = "my-repo";
/// let start_commit = "abcdef";
/// let end_commit = "123456";
///
/// let mut commits_iter = client.compare_commits(project_key, repo_slug, start_commit, end_commit);
/// let all_commits = commits_iter.all().await.unwrap();
///
/// for commit in all_commits {
///     println!("Commit ID: {}", commit.id);
///     println!("Author: {}", commit.author.display_name);
///     println!("Message: {}", commit.message);
/// }
/// ```
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

/// The `BitbucketAuthor` struct represents an author or committer of a commit returned by the Bitbucket API.
///
/// It contains information about the author, such as their name, email address, and display name.
///
/// This struct is usually used as a part of the `BitbucketCommit` struct when working with the `BitbucketClient`.
///
/// # Example
///
/// Suppose you want to fetch all commits between two commit hashes using the `BitbucketClient::compare_commits()` method.
/// You'll receive a `BitbucketPaginated<BitbucketCommit>` iterator, which you can use to fetch all pages of commits:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::{BitbucketClient, BitbucketPaginated};
/// use deployment_changelog::api::rest::Paginated;
///
/// // Suppose you have a BitbucketClient named 'client'
/// let project_key = "PROJECT";
/// let repo_slug = "my-repo";
/// let start_commit = "abcdef";
/// let end_commit = "123456";
///
/// let mut commits_iter = client.compare_commits(project_key, repo_slug, start_commit, end_commit);
/// let all_commits = commits_iter.all().await.unwrap();
///
/// for commit in all_commits {
///     let author = &commit.author;
///     println!("Author name: {}", author.name);
///     println!("Author email: {}", author.email_address);
///     println!("Author display name: {}", author.display_name);
/// }
/// ```
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

/// The `BitbucketPullRequest` struct represents a pull request returned by the Bitbucket API.
///
/// It contains information about the pull request, such as the ID, title, description, open status, author, and creation and update dates.
///
/// This struct is usually used when working with the `BitbucketClient` to fetch pull requests associated with a commit.
///
/// # Example
///
/// Suppose you want to fetch all pull requests associated with a commit hash using the `BitbucketClient::get_pull_requests()` method.
/// You'll receive a `BitbucketPaginated<BitbucketPullRequest>` iterator, which you can use to fetch all pages of pull requests:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::{BitbucketClient, BitbucketPaginated};
/// use deployment_changelog::api::rest::Paginated;
///
/// // Suppose you have a BitbucketClient named 'client'
/// let project_key = "PROJECT";
/// let repo_slug = "my-repo";
/// let commit_hash = "abcdef";
///
/// let mut pr_iter = client.get_pull_requests(project_key, repo_slug, commit_hash);
/// let all_pull_requests = pr_iter.all().await.unwrap();
///
/// for pr in all_pull_requests {
///     println!("Pull request ID: {}", pr.id);
///     println!("Title: {}", pr.title);
///     println!("Description: {}", pr.description);
///     println!("Open: {}", pr.open);
///     println!("Created: {}", pr.created_date);
///     println!("Updated: {}", pr.updated_date);
/// }
/// ```
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
    pub created_date: DateTime<Local>,

    #[serde_as(as = "TimestampMilliSeconds<String, Flexible>")]
    pub updated_date: DateTime<Local>
}

impl Display for BitbucketPullRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Bitbucket pull request: {error}")
        }
    }
}

/// The `BitbucketPullRequestAuthor` struct represents the author of a pull request returned by the Bitbucket API.
///
/// It contains information about the author, such as the user and whether the pull request has been approved by the author.
///
/// This struct is usually used as part of the `BitbucketPullRequest` struct when working with the `BitbucketClient` to fetch pull requests associated with a commit.
///
/// # Example
///
/// Suppose you want to fetch all pull requests associated with a commit hash using the `BitbucketClient::get_pull_requests()` method.
/// You'll receive a `BitbucketPaginated<BitbucketPullRequest>` iterator, which you can use to fetch all pages of pull requests:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::{BitbucketClient, BitbucketPaginated};
/// use deployment_changelog::api::rest::Paginated;
///
/// // Suppose you have a BitbucketClient named 'client'
/// let project_key = "PROJECT";
/// let repo_slug = "my-repo";
/// let commit_hash = "abcdef";
///
/// let mut pr_iter = client.get_pull_requests(project_key, repo_slug, commit_hash);
/// let all_pull_requests = pr_iter.all().await.unwrap();
///
/// for pr in all_pull_requests {
///     println!("Author display name: {}", pr.author.user.display_name);
///     println!("Author email: {}", pr.author.user.email_address);
///     println!("Author approval status: {}", pr.author.approved);
/// }
/// ```
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

/// The `BitbucketPullRequestIssue` struct represents an issue associated with a pull request returned by the Bitbucket API.
///
/// It contains information about the issue, such as the key and URL of the issue.
///
/// This struct is usually used when working with the `BitbucketClient` to fetch issues associated with a specific pull request.
///
/// # Example
///
/// Suppose you want to fetch all issues associated with a pull request using the `BitbucketClient::get_pull_request_issues()` method.
/// You'll receive a `Result<Vec<BitbucketPullRequestIssue>>`, which you can use to access and process the associated issues:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::BitbucketClient;
///
/// // Suppose you have a BitbucketClient named 'client'
/// let project_key = "PROJECT";
/// let repo_slug = "my-repo";
/// let pull_request_id = 42;
///
/// let issues_result = client.get_pull_request_issues(project_key, repo_slug, pull_request_id).await;
///
/// match issues_result {
///     Ok(issues) => {
///         for issue in issues {
///             println!("Issue key: {}", issue.key);
///             println!("Issue URL: {}", issue.url);
///         }
///     },
///     Err(error) => {
///         println!("Error fetching pull request issues: {:?}", error);
///     }
/// }
/// ```
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

/// The `BitbucketClient` struct is a high-level API client for working with the Bitbucket API.
///
/// It provides methods for common operations like comparing commits, fetching pull requests for a commit, and getting issues associated with a pull request.
///
/// Internally, it uses the `RestClient` struct for making API calls.
///
/// # Example
///
/// To create a new `BitbucketClient`, you can use the `new()` method and pass the base URL of your Bitbucket instance:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::BitbucketClient;
///
/// let base_url = "https://bitbucket.example.com";
/// let client = BitbucketClient::new(base_url).unwrap();
/// ```
///
/// Once you have a `BitbucketClient`, you can use it to interact with the Bitbucket API:
///
/// ```rust
/// use deployment_changelog::api::bitbucket::{BitbucketClient, BitbucketCommit};
///
/// // Suppose you have a BitbucketClient named 'client'
/// let project_key = "PROJECT";
/// let repo_slug = "my-repo";
/// let start_commit = "abcdef";
/// let end_commit = "ghijkl";
///
/// let mut commits_paginated = client.compare_commits(project_key, repo_slug, start_commit, end_commit);
///
/// while let Some(commits_result) = commits_paginated.next().await {
///     match commits_result {
///         Ok(commits) => {
///             for commit in commits {
///                 println!("Commit ID: {}", commit.id);
///                 println!("Commit message: {}", commit.message);
///             }
///         },
///         Err(error) => {
///             println!("Error fetching commits: {:?}", error);
///         }
///     }
/// }
/// ```
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
