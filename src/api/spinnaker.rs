//! The `deployment_changelog::api::rest::spinnaker` module provides a client for interacting
//! with the Spinnaker API, specifically for fetching environment states. The client is built
//! on top of the `GraphQLClient` from the `deployment_changelog::api::rest::graphql` module
//! to facilitate communication with the Spinnaker GraphQL API.
//!
//! The `SpinnakerClient` struct provides methods for creating a new client instance
//! with a specified base URL and for fetching environment states. The module also includes
//! the `MdEnvironmentStatesQuery` struct, which is a GraphQL query used for fetching
//! environment states from the Spinnaker API.
//!
//! # Example
//!
//! ```rust
//! use deployment_changelog::api::rest::spinnaker::{SpinnakerClient, md_environment_states_query};
//! use chrono::{DateTime, Local};
//!
//! async fn fetch_environment_states() -> Result<md_environment_states_query::ResponseData> {
//!     let spinnaker_client = SpinnakerClient::new("https://api.example.com")?;
//!
//!     let variables = md_environment_states_query::Variables {
//!         // ... populate variables here ...
//!     };
//!
//!     spinnaker_client.get_environment_states(variables).await
//! }
//! ```
//!
//! In this example, we create a new `SpinnakerClient` instance with the base URL of the
//! Spinnaker API, then call the `get_environment_states` method with the necessary
//! variables to fetch the environment states data. The result is a
//! `md_environment_states_query::ResponseData` object containing the fetched data.
use chrono::{DateTime, Local};
use graphql_client::GraphQLQuery;
use anyhow::{Result, Context, bail};

use super::graphql::GraphQLClient;

type InstantTime = DateTime<Local>;

/// The `MdEnvironmentStatesQuery` struct represents the GraphQL query used to fetch
/// environment states from the Spinnaker API. It is derived from the `GraphQLQuery` trait
/// and contains the required query path, schema path, and response_derives.
///
/// This struct is used internally by the `SpinnakerClient` to execute the environment
/// states query and fetch the data from the Spinnaker API.
///
/// # Example
///
/// Typically, you don't need to interact with `MdEnvironmentStatesQuery` directly,
/// as the `SpinnakerClient` handles the query execution. However, if you want to
/// work with the query directly, you can do so:
///
/// ```rust
/// use deployment_changelog::api::rest::spinnaker::{MdEnvironmentStatesQuery, GraphQLClient};
/// use deployment_changelog::api::rest::graphql::Response;
/// use deployment_changelog::api::rest::spinnaker::md_environment_states_query;
///
/// async fn execute_environment_states_query() -> Result<Response<md_environment_states_query::ResponseData>> {
///     let graphql_client = GraphQLClient::new("https://api.example.com")?;
///
///     let variables = md_environment_states_query::Variables {
///         // ... populate variables here ...
///     };
///
///     graphql_client.post::<MdEnvironmentStatesQuery>(variables).await
/// }
/// ```
///
/// In this example, we create a new `GraphQLClient` instance with the base URL of the
/// Spinnaker API, then call the `post` method with the necessary variables to execute
/// the `MdEnvironmentStatesQuery` and fetch the environment states data.
/// The result is a `Response<md_environment_states_query::ResponseData>` object containing
/// the fetched data.
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "resources/graphql/spinnaker/schema.graphql",
    query_path = "resources/graphql/spinnaker/queries.graphql",
    response_derives = "Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone"
)]
pub struct MdEnvironmentStatesQuery;

/// The `SpinnakerClient` struct provides an interface to interact with the Spinnaker API
/// for fetching environment states. It wraps the `GraphQLClient` and handles the execution
/// of the `MdEnvironmentStatesQuery` for you.
///
/// # Example
///
/// To use the `SpinnakerClient`, first create a new instance with the base URL of the
/// Spinnaker API:
///
/// ```rust
/// use deployment_changelog::api::rest::spinnaker::{SpinnakerClient, md_environment_states_query};
///
/// async fn fetch_environment_states() -> Result<md_environment_states_query::ResponseData> {
///     let spinnaker_client = SpinnakerClient::new("https://api.example.com")?;
///
///     let variables = md_environment_states_query::Variables {
///         // ... populate variables here ...
///     };
///
///     spinnaker_client.get_environment_states(variables).await
/// }
/// ```
///
/// This example demonstrates how to create a new `SpinnakerClient` instance, populate
/// the required variables, and call the `get_environment_states` method to fetch the
/// environment states data from the Spinnaker API.
/// The result is an `md_environment_states_query::ResponseData` object containing
/// the fetched data.
#[derive(Debug)]
pub struct SpinnakerClient {
    client: GraphQLClient
}

impl SpinnakerClient {
    /// Constructs a new `SpinnakerClient` instance with the provided base URL for the Spinnaker API.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Spinnaker API, as a string.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `SpinnakerClient` instance, or an error if there was an issue
    /// creating the underlying `GraphQLClient`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deployment_changelog::api::rest::spinnaker::SpinnakerClient;
    ///
    /// let spinnaker_client = SpinnakerClient::new("https://api.example.com")?;
    /// ```
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: GraphQLClient::new(base_url)?
        })
    }

    /// Constructs a new `SpinnakerClient` instance from an existing `GraphQLClient`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `GraphQLClient` instance.
    ///
    /// # Returns
    ///
    /// A new `SpinnakerClient` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deployment_changelog::api::rest::{spinnaker::SpinnakerClient, graphql::GraphQLClient};
    ///
    /// let graphql_client = GraphQLClient::new("https://api.example.com")?;
    /// let spinnaker_client = SpinnakerClient::from_client(graphql_client);
    /// ```
    pub fn from_client(client: GraphQLClient) -> Self {
        Self {
            client
        }
    }

    /// Fetches environment states data from the Spinnaker API using the provided query variables.
    ///
    /// # Arguments
    ///
    /// * `variables` - An `md_environment_states_query::Variables` instance containing the
    ///   required variables for the `MdEnvironmentStatesQuery`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `md_environment_states_query::ResponseData` if the request was
    /// successful, or an error if there was an issue executing the GraphQL call or processing the
    /// response.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deployment_changelog::api::rest::spinnaker::{SpinnakerClient, md_environment_states_query};
    ///
    /// async fn fetch_environment_states() -> Result<md_environment_states_query::ResponseData> {
    ///     let spinnaker_client = SpinnakerClient::new("https://api.example.com")?;
    ///
    ///     let variables = md_environment_states_query::Variables {
    ///         // ... populate variables here ...
    ///     };
    ///
    ///     spinnaker_client.get_environment_states(variables).await
    /// }
    /// ```
    pub async fn get_environment_states(
        &self,
        variables: md_environment_states_query::Variables
    ) -> Result<md_environment_states_query::ResponseData> {
        let response = self.client.post::<MdEnvironmentStatesQuery>(variables)
            .await?;

        if let Some(errors) = response.errors {
            bail!("Received errors from GraphQL call {:#?}", errors);
        }

        response.data
            .with_context(|| "No data received for GraphQL call but no errors were found")
    }
}
