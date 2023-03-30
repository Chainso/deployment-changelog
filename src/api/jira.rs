//! The `deployment_changelog::api::jira` module provides a high-level API client for interacting with Jira.
//!
//! The main struct in this module is `JiraClient`, which provides methods for common Jira operations, such as fetching an issue by key.
//!
//! Other structs, such as `JiraIssue`, `JiraIssueFields`, `Comments`, and `Comment`, model the Jira data returned by the API.
//!
//! Internally, the `JiraClient` uses the `RestClient` struct for making API calls.
//!
//! # Example
//!
//! To create a new `JiraClient`, you can use the `new()` method and pass the base URL of your Jira instance:
//!
//! ```rust
//! use deployment_changelog::api::jira::JiraClient;
//!
//! let base_url = "https://jira.example.com";
//! let client = JiraClient::new(base_url).unwrap();
//! ```
//!
//! Once you have a `JiraClient`, you can use it to interact with the Jira API:
//!
//! ```rust
//! use deployment_changelog::api::jira::{JiraClient, JiraIssue};
//!
//! // Suppose you have a JiraClient named 'client'
//! let issue_key = "PROJECT-123";
//!
//! match client.get_issue(issue_key).await {
//!     Ok(issue) => {
//!         println!("Issue key: {}", issue.key);
//!         println!("Issue summary: {}", issue.fields.summary);
//!         println!("Issue description: {:?}", issue.fields.description);
//!         println!("Issue comments: {:#?}", issue.fields.comment.comments);
//!     },
//!     Err(error) => {
//!         println!("Error fetching issue: {:?}", error);
//!     }
//! }
//! ```
use std::fmt::Display;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};

use anyhow::Result;

use super::rest::RestClient;

enum JiraEndpoints {
    GetIssue
}

impl JiraEndpoints {
    fn url(&self) -> &'static str {
        match self {
            JiraEndpoints::GetIssue => "rest/api/latest/issue/{issueKey}"
        }
    }
}

/// The `JiraIssue` struct represents a Jira issue and its associated fields.
///
/// # Example
///
/// The following example demonstrates how to use the `JiraClient` to fetch a specific Jira issue and access its properties:
///
/// ```rust
/// use deployment_changelog::api::jira::{JiraClient, JiraIssue};
///
/// async fn get_jira_issue() {
///     let jira_base_url = "https://your-jira-instance.com";
///     let jira_client = JiraClient::new(jira_base_url).unwrap();
///     let issue_key = "DEMO-123";
///
///     let issue: JiraIssue = jira_client.get_issue(issue_key).await.unwrap();
///     println!("Issue key: {}", issue.key);
///     println!("Issue summary: {}", issue.fields.summary);
///     println!("Issue description: {:?}", issue.fields.description);
///     println!("Issue created at: {}", issue.fields.created);
///     println!("Issue updated at: {}", issue.fields.updated);
/// }
/// ```
///
/// You can also easily print the issue as a formatted JSON string:
///
/// ```rust
/// use deployment_changelog::api::jira::{JiraClient, JiraIssue};
///
/// async fn print_jira_issue() {
///     let jira_base_url = "https://your-jira-instance.com";
///     let jira_client = JiraClient::new(jira_base_url).unwrap();
///     let issue_key = "DEMO-123";
///
///     let issue: JiraIssue = jira_client.get_issue(issue_key).await.unwrap();
///     println!("{}", issue); // Outputs the formatted JSON representation of the issue
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssue {
    pub key: String,
    pub fields: JiraIssueFields
}

impl Display for JiraIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Jira issue: {error}")
        }
    }
}

/// The `JiraIssueFields` struct represents the fields associated with a Jira issue.
///
/// # Example
///
/// The following example demonstrates how to use the `JiraClient` to fetch a specific Jira issue and access its fields:
///
/// ```rust
/// use deployment_changelog::api::jira::{JiraClient, JiraIssue};
///
/// async fn get_jira_issue_fields() {
///     let jira_base_url = "https://your-jira-instance.com";
///     let jira_client = JiraClient::new(jira_base_url).unwrap();
///     let issue_key = "DEMO-123";
///
///     let issue: JiraIssue = jira_client.get_issue(issue_key).await.unwrap();
///     let fields = &issue.fields;
///     println!("Issue summary: {}", fields.summary);
///     println!("Issue description: {:?}", fields.description);
///     println!("Issue created at: {}", fields.created);
///     println!("Issue updated at: {}", fields.updated);
///
///     // Iterate through comments
///     for comment in &fields.comment.comments {
///         println!("Comment author: {}", comment.author.display_name);
///         println!("Comment body: {}", comment.body);
///         println!("Comment created at: {}", comment.created);
///         println!("Comment updated at: {}", comment.updated);
///     }
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueFields {
    pub summary: String,
    pub description: Option<String>,
    pub comment: Comments,
    pub created: DateTime<Local>,
    pub updated: DateTime<Local>
}

impl Display for JiraIssueFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Jira issue fields: {error}")
        }
    }
}

/// The `Comments` struct represents a collection of comments associated with a Jira issue.
///
/// # Example
///
/// The following example demonstrates how to use the `JiraClient` to fetch a specific Jira issue and access its comments:
///
/// ```rust
/// use deployment_changelog::api::jira::{JiraClient, JiraIssue};
///
/// async fn get_jira_issue_comments() {
///     let jira_base_url = "https://your-jira-instance.com";
///     let jira_client = JiraClient::new(jira_base_url).unwrap();
///     let issue_key = "DEMO-123";
///
///     let issue: JiraIssue = jira_client.get_issue(issue_key).await.unwrap();
///     let comments = &issue.fields.comment.comments;
///
///     // Iterate through comments
///     for comment in comments {
///         println!("Comment author: {}", comment.author.display_name);
///         println!("Comment body: {}", comment.body);
///         println!("Comment created at: {}", comment.created);
///         println!("Comment updated at: {}", comment.updated);
///     }
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Comments {
    pub comments: Vec<Comment>
}

impl Display for Comments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Jira comments: {error}")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub author: JiraAuthor,
    pub body: String,
    pub created: DateTime<Local>,
    pub updated: DateTime<Local>
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Jira comment: {error}")
        }
    }
}

/// The `JiraAuthor` struct represents the author of a comment or other content within a Jira issue.
///
/// # Example
///
/// The following example demonstrates how to use the `JiraClient` to fetch a specific Jira issue and access the authors of its comments:
///
/// ```rust
/// use deployment_changelog::api::jira::{JiraClient, JiraIssue};
///
/// async fn get_jira_issue_comment_authors() {
///     let jira_base_url = "https://your-jira-instance.com";
///     let jira_client = JiraClient::new(jira_base_url).unwrap();
///     let issue_key = "DEMO-123";
///
///     let issue: JiraIssue = jira_client.get_issue(issue_key).await.unwrap();
///     let comments = &issue.fields.comment.comments;
///
///     // Iterate through comments
///     for comment in comments {
///         let author = &comment.author;
///         println!("Author name: {}", author.name);
///         println!("Author key: {}", author.key);
///         println!("Author display name: {}", author.display_name);
///     }
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct JiraAuthor {
    pub name: String,
    pub key: String,
    pub display_name: String
}

impl Display for JiraAuthor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Jira author: {error}")
        }
    }
}

/// The `JiraClient` struct provides a high-level interface to interact with the Jira REST API. It includes methods for fetching Jira issues and working with their data.
///
/// # Example
///
/// The following example demonstrates how to create a `JiraClient` instance, fetch a specific Jira issue, and print its summary and description:
///
/// ```rust
/// use deployment_changelog::api::jira::{JiraClient, JiraIssue};
///
/// async fn print_jira_issue_summary_and_description() {
///     let jira_base_url = "https://your-jira-instance.com";
///     let jira_client = JiraClient::new(jira_base_url).unwrap();
///     let issue_key = "DEMO-123";
///
///     let issue: JiraIssue = jira_client.get_issue(issue_key).await.unwrap();
///     println!("Issue summary: {}", issue.fields.summary);
///     println!("Issue description: {:?}", issue.fields.description);
/// }
/// ```
pub struct JiraClient {
    client: RestClient
}

impl JiraClient {
    /// Creates a new `JiraClient` instance with the specified Jira base URL.
    ///
    /// # Example
    ///
    /// ```
    /// use deployment_changelog::api::jira::JiraClient;
    ///
    /// let jira_base_url = "https://your-jira-instance.com";
    /// let jira_client = JiraClient::new(jira_base_url).unwrap();
    /// ```
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: RestClient::new(base_url)?
        })
    }

    /// Creates a new `JiraClient` instance from an existing `RestClient` instance.
    ///
    /// # Example
    ///
    /// ```
    /// use deployment_changelog::api::jira::JiraClient;
    /// use deployment_changelog::api::rest::RestClient;
    ///
    /// let jira_base_url = "https://your-jira-instance.com";
    /// let rest_client = RestClient::new(jira_base_url).unwrap();
    /// let jira_client = JiraClient::from_client(rest_client);
    /// ```
    pub fn from_client(client: RestClient) -> Self {
        Self {
            client
        }
    }

    /// Fetches a Jira issue with the specified issue key.
    ///
    /// # Example
    ///
    /// ```
    /// use deployment_changelog::api::jira::{JiraClient, JiraIssue};
    ///
    /// async fn fetch_jira_issue() {
    ///     let jira_base_url = "https://your-jira-instance.com";
    ///     let jira_client = JiraClient::new(jira_base_url).unwrap();
    ///     let issue_key = "DEMO-123";
    ///
    ///     let issue: JiraIssue = jira_client.get_issue(issue_key).await.unwrap();
    ///     println!("Fetched issue: {:?}", issue);
    /// }
    /// ```
    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue> {
        let issue_path: String = JiraEndpoints::GetIssue.url()
            .replace("{issueKey}", issue_key);

        self.client.get::<JiraIssue>(&issue_path, None).await
    }
}
