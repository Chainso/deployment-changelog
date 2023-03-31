//! A module for providing easy-to-use clients to deal with external APIs
pub mod rest;
pub mod graphql;
pub mod bitbucket;
pub mod jira;

#[cfg(feature = "spinnaker")]
pub mod spinnaker;
