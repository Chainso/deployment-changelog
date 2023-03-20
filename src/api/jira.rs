use std::fmt::Display;

use serde::{Serialize, Deserialize};
use anyhow::Result;

use super::api::RestClient;

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueFields {
    pub summary: String,
    pub description: String,
    pub comment: Comments
}

impl Display for JiraIssueFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Jira issue fields: {error}")
        }
    }
}

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
    pub body: String
    // pub created: Date,
    // pub updated: Date
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => write!(f, "{json}"),
            Err(error) => panic!("Error serializing Jira comment: {error}")
        }
    }
}

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

pub struct JiraClient {
    client: RestClient
}

impl JiraClient {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: RestClient::new(base_url)?
        })
    }

    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue> {
        let issue_path: String = JiraEndpoints::GetIssue.url()
            .replace("{issueKey}", issue_key);

        self.client.get::<JiraIssue>(&issue_path, None).await
    }
}
