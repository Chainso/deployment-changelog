# Deployment Changelog

[![Crates.io](https://img.shields.io/crates/v/deployment_changelog.svg)](https://crates.io/crates/deployment_changelog)
[![Docs.rs](https://docs.rs/deployment_changelog/badge.svg)](https://docs.rs/deployment_changelog)

A Rust library for generating changelogs based on deployments or commit ranges in Git repositories. It integrates with Bitbucket, Jira, and Spinnaker to retrieve commit, pull request, and issue information for the specified range or Spinnaker environment. The crate provides a structured `Changelog` object with a human-readable display representation.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Examples](#examples)
- [API Documentation](#api-documentation)
- [Contributing](#contributing)
- [License](#license)

## Features

- Fetch commits, pull requests, and issues in a Git commit range.
- Generate changelogs for Spinnaker environments.
- Integrate with Bitbucket, Jira, and Spinnaker APIs.
- Serialize and display changelogs in a human-readable format.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
deployment_changelog = "0.1.0"
```

# Usage

To generate a changelog, you can either use a Git commit range or a Spinnaker environment.

## Examples

Changelog from a Git commit range

```rust
use deployment_changelog::changelog::{Changelog, GitCommitRange};
use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient};

// Create a BitbucketClient and JiraClient with their respective server URLs.
let bitbucket_client = BitbucketClient::new("https://your-bitbucket-url");
let jira_client = JiraClient::new("https://your-jira-url");

// Define the Git commit range for the changelog.
let commit_range = GitCommitRange {
    project: String::from("my-project"),
    repo: String::from("my-repo"),
    start_commit: String::from("abcdef123456"),
    end_commit: String::from("ghijkl789012")
};

// Generate a Changelog using the get_changelog_from_range method and print the formatted output.
let changelog = Changelog::get_changelog_from_range(&bitbucket_client, &jira_client, &commit_range).await.unwrap();
println!("{}", changelog);
```

Changelog from a Spinnaker environment

```rust
use deployment_changelog::changelog::{Changelog, CommitSpecifier, SpinnakerEnvironment};
use deployment_changelog::api::{bitbucket::BitbucketClient, jira::JiraClient, spinnaker::SpinnakerClient};

// Create a BitbucketClient, JiraClient, and SpinnakerClient with their respective server URLs.
let bitbucket_client = BitbucketClient::new("https://your-bitbucket-url");
let jira_client = JiraClient::new("https://your-jira-url");
let spinnaker_client = SpinnakerClient::new("https://your-spinnaker-url");

// Define the Spinnaker environment for the changelog.
let spinnaker_env = SpinnakerEnvironment {
    client: spinnaker_client,
    app_name: String::from("my-app"),
    env: String::from("my-environment")
};

// Create a CommitSpecifier using the Spinnaker environment.
let commit_specifier = CommitSpecifier::Spinnaker(spinnaker_env);

// Generate a Changelog using the get_changelog_from_spinnaker method and print the formatted output.
let changelog = Changelog::new(&bitbucket_client, &jira_client, &commit_specifier).await.unwrap();
println!("{}", changelog);
```

## Prerequisites

Before using the `deployment_changelog` crate, make sure you have access to the following services:

- Bitbucket
- Jira
- Spinnaker [Optional]

Additionally, you'll need to generate API tokens or credentials for each service.

## Configuration

To use the `deployment_changelog` crate, you need to configure the `BitbucketClient`, `JiraClient`, and `SpinnakerClient` with the necessary API tokens or credentials. Custom TLS certificates and authorization headers can be added to the underlying `reqwest` client.

### Basic Configuration

```rust
let bitbucket_client = BitbucketClient::new("https://your-bitbucket-url");
let jira_client = JiraClient::new("https://your-jira-url");
let spinnaker_client = SpinnakerClient::new("https://your-spinnaker-url");
```

### Custom Configuration with reqwest

For more advanced configuration options, such as custom TLS certificates or custom authorization headers, you can modify the underlying reqwest client used by the BitbucketClient, JiraClient, and SpinnakerClient. Here's an example of how to configure the clients with custom TLS certificates and authorization headers:

```rust
use deployment_changelog::api::{
    rest::{RestClient, RestClientBuilder},
    bitbucket::BitbucketClient,
};
use reqwest::{ClientBuilder, header::{HeaderMap, HeaderValue, AUTHORIZATION}};

let mut headers = HeaderMap::new();
headers.insert(AUTHORIZATION, HeaderValue::from_static("your-custom-authorization-header"));

let mut client_builder = RestClient::builder("https://your-bitbucket-url");
client_builder.client_builder = client_builder.client_builder
    .default_headers(headers)
    .danger_accept_invalid_certs(true); // Use this option with caution, only in a trusted environment.

let bitbucket_client = BitbucketClient::from_client(client.build()?);
```

## CLI Usage

The `deployment_changelog` crate also includes a simple command-line interface (CLI) for generating deployment changelogs from public Bitbucket and JIRA instances. To use the CLI, you can clone the repository.

This CLI tool accepts arguments for specifying the Bitbucket and JIRA servers, as well as commit specifier details like Spinnaker environment or Git commit range. You can build and run the CLI tool with `cargo run`, providing the required arguments.

Here's an example of how to use the CLI tool:

### Spinnaker specifier

```sh
export BITBUCKET_URL=https://your-bitbucket-url.com/
export JIRA_URL=https://your-jira-url.com/
export SPINNAKER_URL=https://your-spinnaker-url.com/

cargo run --all-features spinnaker important_service prod
```

### Git commit range specifier

```sh
export BITBUCKET_URL=https://your-bitbucket-url.com/
export JIRA_URL=https://your-jira-url.com/

cargo run commit-range CATS clowder abc123def4567890a1b2c3d4e5f67890abcdef01 5f56c43386103d10c1cbb415d6f3132da16948a8
```

The CLI will output the changelog in the console.

# Version Compatibility

The deployment_changelog crate requires Rust 1.53.0 or later.

# API Documentation

Find detailed API documentation on [docs.rs](https://docs.rs/deployment_changelog).

# Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request if you have any ideas or improvements.

# License

This project is licensed under the MIT License.

