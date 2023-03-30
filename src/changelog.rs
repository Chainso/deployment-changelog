use crate::api::{rest::Paginated, jira::{JiraIssue, JiraClient}, bitbucket::{BitbucketCommit, BitbucketPullRequest, BitbucketPullRequestIssue, BitbucketClient, BitbucketPaginated}};

use std::{fmt::Display, collections::HashSet};
use clap::Parser;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Parser, Debug)]
pub enum CommitSpecifier {
    Spinnaker(SpinnakerArgs),
    CommitRange(CommitRange)
}

#[derive(Parser, Debug)]
pub struct SpinnakerArgs {
    #[clap(long, short = 's', about, long_help = "The URL to your Spinnaker server", env = "SPINNAKER_URL")]
    url: String,
    env: String
}

#[derive(Parser, Debug)]
pub struct CommitRange {
    start_commit: String,
    end_commit: String
}

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

impl Changelog {
    pub async fn new(
        bitbucket_client: &BitbucketClient,
        jira_client: &JiraClient,
        project: &str,
        repo: &str,
        commit_specifier: &CommitSpecifier
    ) -> Result<Changelog> {
        match commit_specifier {
            CommitSpecifier::Spinnaker(spinnaker_args) => unimplemented!(),
            CommitSpecifier::CommitRange(commit_range) => Self::get_changelog_from_range(
                bitbucket_client,
                jira_client,
                project,
                repo,
                commit_range
            ).await
        }
    }

    pub async fn get_changelog_from_spinnaker(
        bitbucket_client: &BitbucketClient,
        jira_client: &JiraClient,
        project: &str,
        repo: &str,
        spinnaker_args: &SpinnakerArgs
    ) -> Result<Changelog> {
        unimplemented!()
    }

    pub async fn get_changelog_from_range(
        bitbucket_client: &BitbucketClient,
        jira_client: &JiraClient,
        project: &str,
        repo: &str,
        commit_range: &CommitRange
    ) -> Result<Changelog> {
        let commits: Vec<BitbucketCommit> = bitbucket_client.compare_commits(
            project,
            repo,
            &commit_range.start_commit,
            &commit_range.end_commit
        )
            .all()
            .await?;

        let mut pull_request_pages: Vec<BitbucketPaginated<BitbucketPullRequest>> = commits.iter()
                .map(|commit| bitbucket_client.get_pull_requests(project, repo, &commit.id))
                .collect();

        let pull_requests: Vec<BitbucketPullRequest> = futures::future::join_all(
            pull_request_pages.iter_mut()
                .map(|page| page.all())
        )
            .await
            .into_iter()
            .collect::<Result<Vec<Vec<BitbucketPullRequest>>>>()?
            .into_iter()
            .flatten()
            .collect::<HashSet<BitbucketPullRequest>>()
            .into_iter()
            .collect();

        let pull_request_issues: Vec<BitbucketPullRequestIssue> = futures::future::join_all(
            pull_requests.iter()
                .map(|pull_request| bitbucket_client.get_pull_request_issues(project, repo, pull_request.id))
        )
            .await
            .into_iter()
            .collect::<Result<Vec<Vec<BitbucketPullRequestIssue>>>>()?
            .into_iter()
            .flatten()
            .collect::<HashSet<BitbucketPullRequestIssue>>()
            .into_iter()
            .collect();

        let issues = futures::future::join_all(
            pull_request_issues.iter()
                .map(|pull_request_issue| jira_client.get_issue(&pull_request_issue.key))
        )
            .await
            .into_iter()
            .collect::<Result<Vec<JiraIssue>>>()?;

        Ok(Changelog {
            commits,
            pull_requests,
            issues
        })
    }
}

