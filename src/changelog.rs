//! The `changelog` module provides functionality for generating a changelog for a deployment
//! based on Jira issues and associated commits and pull requests in Bitbucket.
//!
//! This module contains the main `Changelog` struct and associated implementations. The `Changelog` struct
//! represents the final changelog data that includes information about commits, pull requests, and Jira issues.
//!
//! # Example
//!
//! ```
//! use deployment_changelog::changelog::{Changelog, CommitSpecifier, GitCommitRange};
//! use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient};
//!
//! #[tokio::main]
//! async fn main() {
//!     let bitbucket_client = BitbucketClient::new("https://api.bitbucket.org");
//!     let jira_client = JiraClient::new("https://your-domain.atlassian.net");
//!
//!     let commit_range = GitCommitRange {
//!         project: String::from("my-project"),
//!         repo: String::from("my-repo"),
//!         start_commit: String::from("abcdef123456"),
//!         end_commit: String::from("ghijkl789012")
//!     };
//!
//!     let commit_specifier = CommitSpecifier::CommitRange(commit_range);
//!
//!     let changelog = Changelog::new(&bitbucket_client, &jira_client, &commit_specifier).await.unwrap();
//!
//!     println!("{:?}", changelog);
//! }
//! ```
//!
//! In this example, we create instances of `BitbucketClient` and `JiraClient` with their
//! respective server URLs. Then, we define a `GitCommitRange` with the `project`, `repo`,
//! `start_commit`, and `end_commit` that we want to generate a changelog for.
//!
//! We use the `GitCommitRange` to create a `CommitSpecifier` and pass it to `Changelog::new` to create
//! a changelog. Finally, we print the changelog.
use crate::api::{rest::Paginated, jira::{JiraIssue, JiraClient}, bitbucket::{BitbucketCommit, BitbucketPullRequest, BitbucketPullRequestIssue, BitbucketClient, BitbucketPaginated}};
use crate::api::spinnaker::{SpinnakerClient, md_environment_states_query::{Variables, MdArtifactStatusInEnvironment, MdEnvironmentStatesQueryApplicationEnvironmentsStateArtifactsVersions}};

use std::{fmt::Display, collections::{HashSet, HashMap}};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

/// The `CommitSpecifier` enum is used to specify the range of commits for which the changelog
/// should be generated. It has two variants: `Spinnaker` and `CommitRange`.
///
/// - `Spinnaker`: This variant uses the `SpinnakerEnvironment` struct to determine the commit range.
///   It fetches the latest pending and current versions from a Spinnaker environment to compute the
///   range of commits for which the changelog should be generated.
///
/// - `CommitRange`: This variant uses the `GitCommitRange` struct to directly specify the range of
///   commits for which the changelog should be generated.
///
/// # Example
///
/// ```
/// use deployment_changelog::changelog::{CommitSpecifier, SpinnakerEnvironment, GitCommitRange};
/// use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient, spinnaker::SpinnakerClient};
///
/// // Creating a CommitSpecifier using the Spinnaker variant
/// let spinnaker_client = SpinnakerClient::new("https://your-spinnaker-url");
/// let spinnaker_env = SpinnakerEnvironment {
///     client: spinnaker_client,
///     app_name: String::from("my-app"),
///     env: String::from("production")
/// };
/// let commit_specifier_spinnaker = CommitSpecifier::Spinnaker(spinnaker_env);
///
/// // Creating a CommitSpecifier using the CommitRange variant
/// let commit_range = GitCommitRange {
///     project: String::from("my-project"),
///     repo: String::from("my-repo"),
///     start_commit: String::from("abcdef123456"),
///     end_commit: String::from("ghijkl789012")
/// };
/// let commit_specifier_range = CommitSpecifier::CommitRange(commit_range);
/// ```
///
/// In this example, we demonstrate how to create instances of `CommitSpecifier` using both the
/// `Spinnaker` and `CommitRange` variants. We create a `SpinnakerEnvironment` struct and a
/// `GitCommitRange` struct and use them to create `CommitSpecifier` instances.
#[derive(Debug)]
pub enum CommitSpecifier {
    Spinnaker(SpinnakerEnvironment),
    CommitRange(GitCommitRange)
}

/// The `SpinnakerEnvironment` struct is used to represent a Spinnaker environment for which the
/// changelog should be generated. It contains the following fields:
///
/// - `client`: A `SpinnakerClient` instance used to interact with the Spinnaker API.
/// - `app_name`: A `String` representing the name of the Spinnaker application.
/// - `env`: A `String` representing the name of the Spinnaker environment (e.g., "production").
///
/// When the `CommitSpecifier::Spinnaker` variant is used, the changelog is generated based on
/// the latest pending and current versions of the specified Spinnaker environment.
///
/// # Example
///
/// ```
/// use deployment_changelog::changelog::{CommitSpecifier, SpinnakerEnvironment};
/// use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient, spinnaker::SpinnakerClient};
///
/// let spinnaker_client = SpinnakerClient::new("https://your-spinnaker-url");
/// let spinnaker_env = SpinnakerEnvironment {
///     client: spinnaker_client,
///     app_name: String::from("my-app"),
///     env: String::from("production")
/// };
/// let commit_specifier = CommitSpecifier::Spinnaker(spinnaker_env);
/// ```
///
/// In this example, we create a `SpinnakerClient` with the Spinnaker server URL, and then create
/// a `SpinnakerEnvironment` instance with the client, application name, and environment name.
/// Finally, we use the `SpinnakerEnvironment` to create a `CommitSpecifier` instance with the
/// `Spinnaker` variant.
#[derive(Debug)]
pub struct SpinnakerEnvironment {
    pub client: SpinnakerClient,
    pub app_name: String,
    pub env: String
}

/// The `GitCommitRange` struct is used to represent a range of commits for which the
/// changelog should be generated. It contains the following fields:
///
/// - `project`: A `String` representing the name of the project in the Git repository.
/// - `repo`: A `String` representing the name of the Git repository.
/// - `start_commit`: A `String` representing the starting commit in the range.
/// - `end_commit`: A `String` representing the ending commit in the range.
///
/// When the `CommitSpecifier::CommitRange` variant is used, the changelog is generated based on
/// the specified range of commits directly.
///
/// # Example
///
/// ```
/// use deployment_changelog::changelog::{CommitSpecifier, GitCommitRange};
/// use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient};
///
/// let commit_range = GitCommitRange {
///     project: String::from("my-project"),
///     repo: String::from("my-repo"),
///     start_commit: String::from("abcdef123456"),
///     end_commit: String::from("ghijkl789012")
/// };
/// let commit_specifier = CommitSpecifier::CommitRange(commit_range);
/// ```
///
/// In this example, we create a `GitCommitRange` instance with the project name, repository name,
/// and starting and ending commit hashes. Then, we use the `GitCommitRange` to create a
/// `CommitSpecifier` instance with the `CommitRange` variant.
#[derive(Debug)]
pub struct GitCommitRange {
    pub project: String,
    pub repo: String,
    pub start_commit: String,
    pub end_commit: String
}

/// The `Changelog` struct represents a changelog containing information about commits,
/// pull requests, and issues between two versions of a project. It contains the following fields:
///
/// - `commits`: A `Vec<BitbucketCommit>` containing the list of Bitbucket commits.
/// - `pull_requests`: A `Vec<BitbucketPullRequest>` containing the list of Bitbucket pull requests.
/// - `issues`: A `Vec<JiraIssue>` containing the list of Jira issues.
///
/// The `Changelog` struct provides methods to generate a changelog from a Spinnaker environment
/// or a Git commit range. It also implements the `Display` trait to provide a formatted output.
///
/// # Example
///
/// ```
/// use deployment_changelog::changelog::{Changelog, CommitSpecifier, GitCommitRange};
/// use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient};
///
/// let bitbucket_client = BitbucketClient::new("https://your-bitbucket-url");
/// let jira_client = JiraClient::new("https://your-jira-url");
///
/// let commit_range = GitCommitRange {
///     project: String::from("my-project"),
///     repo: String::from("my-repo"),
///     start_commit: String::from("abcdef123456"),
///     end_commit: String::from("ghijkl789012")
/// };
///
/// let commit_specifier = CommitSpecifier::CommitRange(commit_range);
/// let changelog = Changelog::new(&bitbucket_client, &jira_client, &commit_specifier).await.unwrap();
///
/// println!("{}", changelog);
/// ```
///
/// In this example, we create a `BitbucketClient` and a `JiraClient` with their respective server URLs.
/// We also create a `GitCommitRange` instance and use it to create a `CommitSpecifier` with the
/// `CommitRange` variant. Then, we generate a `Changelog` using the `Changelog::new` method and
/// print the formatted output.
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
    /// This method creates a new `Changelog` instance using the provided `BitbucketClient`, `JiraClient`,
    /// and `CommitSpecifier`. The changelog is generated based on the commit specifier. It can either
    /// generate a changelog from a Spinnaker environment or a Git commit range.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use deployment_changelog::changelog::{Changelog, CommitSpecifier, GitCommitRange};
    /// use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient};
    ///
    /// // Create a BitbucketClient and JiraClient with their respective server URLs.
    /// let bitbucket_client = BitbucketClient::new("https://your-bitbucket-url");
    /// let jira_client = JiraClient::new("https://your-jira-url");
    ///
    /// // Define the Git commit range for the changelog.
    /// let commit_range = GitCommitRange {
    ///     project: String::from("my-project"),
    ///     repo: String::from("my-repo"),
    ///     start_commit: String::from("abcdef123456"),
    ///     end_commit: String::from("ghijkl789012")
    /// };
    ///
    /// // Create a CommitSpecifier using the Git commit range.
    /// let commit_specifier = CommitSpecifier::CommitRange(commit_range);
    ///
    /// // Generate a Changelog using the new method and print the formatted output.
    /// let changelog = Changelog::new(&bitbucket_client, &jira_client, &commit_specifier).await.unwrap();
    /// println!("{}", changelog);
    /// ```
    ///
    /// In this example, we create a `BitbucketClient` and a `JiraClient` with their respective server URLs.
    /// We define a `GitCommitRange` instance and use it to create a `CommitSpecifier` with the
    /// `CommitRange` variant. Then, we generate a `Changelog` using the `Changelog::new` method and
    /// print the formatted output.
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

    /// This method creates a `Changelog` instance for a Spinnaker environment. It fetches the
    /// environment's latest pending and current versions and generates a changelog based on the
    /// commit range between these two versions.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use deployment_changelog::changelog::{Changelog, CommitSpecifier, SpinnakerEnvironment};
    /// use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient, spinnaker::SpinnakerClient};
    ///
    /// // Create a BitbucketClient, JiraClient, and SpinnakerClient with their respective server URLs.
    /// let bitbucket_client = BitbucketClient::new("https://your-bitbucket-url");
    /// let jira_client = JiraClient::new("https://your-jira-url");
    /// let spinnaker_client = SpinnakerClient::new("https://your-spinnaker-url");
    ///
    /// // Define the Spinnaker environment for the changelog.
    /// let spinnaker_env = SpinnakerEnvironment {
    ///     client: spinnaker_client,
    ///     app_name: String::from("my-app"),
    ///     env: String::from("my-environment")
    /// };
    ///
    /// // Create a CommitSpecifier using the Spinnaker environment.
    /// let commit_specifier = CommitSpecifier::Spinnaker(spinnaker_env);
    ///
    /// // Generate a Changelog using the get_changelog_from_spinnaker method and print the formatted output.
    /// let changelog = Changelog::get_changelog_from_spinnaker(&bitbucket_client, &jira_client, &spinnaker_env).await.unwrap();
    /// println!("{}", changelog);
    /// ```
    ///
    /// In this example, we create a `BitbucketClient`, a `JiraClient`, and a `SpinnakerClient` with their respective server URLs.
    /// We define a `SpinnakerEnvironment` instance and use it to create a `CommitSpecifier` with the
    /// `Spinnaker` variant. Then, we generate a `Changelog` using the `Changelog::get_changelog_from_spinnaker` method and
    /// print the formatted output.
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

    /// This method creates a `Changelog` instance for a specified Git commit range. It fetches
    /// the commits, pull requests, and issues in the range and generates a changelog based on
    /// the collected data.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use deployment_changelog::changelog::{Changelog, GitCommitRange};
    /// use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient};
    ///
    /// // Create a BitbucketClient and JiraClient with their respective server URLs.
    /// let bitbucket_client = BitbucketClient::new("https://your-bitbucket-url");
    /// let jira_client = JiraClient::new("https://your-jira-url");
    ///
    /// // Define the Git commit range for the changelog.
    /// let commit_range = GitCommitRange {
    ///     project: String::from("my-project"),
    ///     repo: String::from("my-repo"),
    ///     start_commit: String::from("abcdef123456"),
    ///     end_commit: String::from("ghijkl789012")
    /// };
    ///
    /// // Generate a Changelog using the get_changelog_from_range method and print the formatted output.
    /// let changelog = Changelog::get_changelog_from_range(&bitbucket_client, &jira_client, &commit_range).await.unwrap();
    /// println!("{}", changelog);
    /// ```
    ///
    /// In this example, we create a `BitbucketClient` and a `JiraClient` with their respective server URLs.
    /// We define a `GitCommitRange` instance and use it to generate a `Changelog` with the
    /// `Changelog::get_changelog_from_range` method. Then, we print the formatted output.
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

