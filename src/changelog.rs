use crate::api::{rest::Paginated, jira::{JiraIssue, JiraClient}, bitbucket::{BitbucketCommit, BitbucketPullRequest, BitbucketPullRequestIssue, BitbucketClient, BitbucketPaginated}, spinnaker::{SpinnakerClient, md_environment_states_query::{Variables, MdArtifactStatusInEnvironment, MdEnvironmentStatesQueryApplicationEnvironmentsStateArtifactsVersions}}};

use std::{fmt::Display, collections::{HashSet, HashMap}};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

#[derive(Debug)]
pub enum CommitSpecifier {
    Spinnaker(SpinnakerEnvironment),
    CommitRange(GitCommitRange)
}

#[derive(Debug)]
pub struct SpinnakerEnvironment {
    pub client: SpinnakerClient,
    pub app_name: String,
    pub env: String
}

#[derive(Debug)]
pub struct GitCommitRange {
    pub project: String,
    pub repo: String,
    pub start_commit: String,
    pub end_commit: String
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
        commit_specifier: &CommitSpecifier
    ) -> Result<Changelog> {
        match commit_specifier {
            CommitSpecifier::Spinnaker(spinnaker_env) => Self::get_changelog_from_spinnaker(
                bitbucket_client,
                jira_client,
                spinnaker_env
            ).await,
            CommitSpecifier::CommitRange(commit_range) => Self::get_changelog_from_range(
                bitbucket_client,
                jira_client,
                commit_range
            ).await
        }
    }

    pub async fn get_changelog_from_spinnaker(
        bitbucket_client: &BitbucketClient,
        jira_client: &JiraClient,
        spinnaker_env: &SpinnakerEnvironment
    ) -> Result<Changelog> {
        let env_state_vars = Variables {
            app_name: spinnaker_env.app_name.clone(),
            environments: vec![spinnaker_env.env.clone()]
        };

        let env_states = spinnaker_env.client.get_environment_states(env_state_vars)
            .await?;

        let application = env_states.application
            .with_context(|| format!("Spinnaker application {} was not found", spinnaker_env.app_name))?;

        let environment = application.environments
            .into_iter()
            .next()
            .with_context(|| format!("Spinnaker application {} has no environment {}", spinnaker_env.app_name, spinnaker_env.env))?;


        let artifacts = environment.state
            .artifacts
            .with_context(|| format!("No artifacts found for environment {} in Spinnaker application {}", spinnaker_env.env, spinnaker_env.app_name))?;

        let mut version_map = HashMap::<MdArtifactStatusInEnvironment, Vec<MdEnvironmentStatesQueryApplicationEnvironmentsStateArtifactsVersions>>::with_capacity(1);

        artifacts.into_iter()
            .for_each(|artifact| {
                if let Some(versions) = artifact.versions {
                    versions.into_iter()
                        .for_each(|version| {
                            if let Some(status) = &version.status {
                                version_map.entry(status.clone())
                                    .or_insert_with(Vec::new)
                                    .push(version);
                            }
                        });
                }
            });

        let pending_versions = version_map.remove(&MdArtifactStatusInEnvironment::PENDING)
            .with_context(|| format!("There are no pending versions for environment {} in Spinnaker application {}", spinnaker_env.env, spinnaker_env.app_name))?;

        let current_versions = version_map.remove(&MdArtifactStatusInEnvironment::CURRENT)
            .with_context(|| format!("There are no current versions for environment {} in Spinnaker application {}", spinnaker_env.env, spinnaker_env.app_name))?;

        let latest_pending_version = pending_versions.into_iter()
            .max_by_key(|version| version.build_number.clone())
            .expect("Error getting latest pending version");

        let latest_current_version = current_versions.into_iter()
            .max_by_key(|version| version.build_number.clone())
            .expect("Error getting latest current version");

        let pending_git_metadata = latest_pending_version.git_metadata
            .with_context(|| format!(
                "Error getting Git metadata for the latest pending version for Spinnaker application {}, environment {}",
                spinnaker_env.app_name,
                spinnaker_env.env)
            )?;

        let current_git_metadata = latest_current_version.git_metadata
            .with_context(|| format!(
                "Error getting Git metadata for the latest current version for Spinnaker application {}, environment {}",
                spinnaker_env.app_name,
                spinnaker_env.env)
            )?;

        let project = pending_git_metadata.project
            .with_context(|| format!(
                "Error getting the Git project for the latest pending version for Spinnaker application {}, environment {}",
                spinnaker_env.app_name,
                spinnaker_env.env)
            )?;

        let repo = pending_git_metadata.repo_name
            .with_context(|| format!(
                "Error getting the Git repository name for latest pending version for Spinnaker application {}, environment {}",
                spinnaker_env.app_name,
                spinnaker_env.env)
            )?;

        let start_commit = pending_git_metadata.commit
            .with_context(|| format!(
                "Error getting the Git commit for the latest pending version for Spinnaker application {}, environment {}",
                spinnaker_env.app_name,
                spinnaker_env.env)
            )?;

        let end_commit = current_git_metadata.commit
            .with_context(|| format!(
                "Error getting the Git commit for the latest current version for Spinnaker application {}, environment {}",
                spinnaker_env.app_name,
                spinnaker_env.env)
            )?;

        let commit_range = GitCommitRange {
            project,
            repo,
            start_commit,
            end_commit
        };

        Self::get_changelog_from_range(
            bitbucket_client,
            jira_client,
            &commit_range
        ).await
    }

    pub async fn get_changelog_from_range(
        bitbucket_client: &BitbucketClient,
        jira_client: &JiraClient,
        commit_range: &GitCommitRange
    ) -> Result<Changelog> {
        let commits: Vec<BitbucketCommit> = bitbucket_client.compare_commits(
            &commit_range.project,
            &commit_range.repo,
            &commit_range.start_commit,
            &commit_range.end_commit
        )
            .all()
            .await?;

        let mut pull_request_pages: Vec<BitbucketPaginated<BitbucketPullRequest>> = commits.iter()
                .map(|commit| bitbucket_client.get_pull_requests(&commit_range.project, &commit_range.repo, &commit.id))
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
                .map(|pull_request| bitbucket_client.get_pull_request_issues(&commit_range.project, &commit_range.repo, pull_request.id))
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

