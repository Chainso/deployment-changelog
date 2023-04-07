//! # deployment_changelog::api::rest::graphql
//!
//! This module provides a `GraphQLClient` for communicating with a GraphQL API endpoint.
//!
//! It leverages the `RestClient` implementation from the `deployment_changelog::api::rest` module to handle
//! HTTP requests and responses, and the `graphql_client` crate to provide type-safe GraphQL query execution.
//!
//! The `GraphQLClient` struct provides a simple interface for executing GraphQL queries and
//! handling their responses. It can be initialized with a base URL and supports executing queries
//! using the `post` method.
//!
//! # Example
//!
//! Below is an example of how to use the `GraphQLClient` to execute a query:
//!
//! ```rust
//! use anyhow::Result;
//! use deployment_changelog::api::rest::graphql::GraphQLClient;
//! use graphql_client::{GraphQLQuery, Response};
//!
//! // Define a query using the graphql_client macro.
//! #[derive(GraphQLQuery)]
//! #[graphql(
//!     schema_path = "path/to/schema.graphql",
//!     query_path = "path/to/query.graphql",
//!     response_derives = "Debug"
//! )]
//! struct MyQuery;
//!
//! async fn execute_query() -> Result<Response<MyQuery::ResponseData>> {
//!     // Create a new GraphQLClient instance.
//!     let graphql_client = GraphQLClient::new("https://api.example.com")?;
//!
//!     // Set the query variables.
//!     let variables = MyQuery::Variables { /* ... */ };
//!
//!     // Execute the query and return the result.
//!     graphql_client.post(variables).await
//! }
//! ```
//!
//! # Errors
//!
//! Errors are handled using the `anyhow` crate, and the `Result` type is used to return errors from
//! functions. The `post` method can return errors related to HTTP requests, response handling, or
//! GraphQL-specific issues.
//!
//! For more detailed examples and further documentation, please refer to the individual struct and method
//! documentation.
use anyhow::{Context, Result};
use graphql_client::{GraphQLQuery, QueryBody, Response};

use super::rest::RestClient;

const GRAPHQL_ENDPOINT: &str = "graphql";

/// A GraphQL client for communicating with a GraphQL API endpoint.
///
/// `GraphQLClient` provides an easy way to execute GraphQL queries and handle their responses.
/// It is built on top of the `RestClient` from the `deployment_changelog::api::rest` module to handle
/// HTTP requests and uses the `graphql_client` crate for type-safe query execution.
///
/// # Example
///
/// Below is an example of how to use the `GraphQLClient` to execute a query:
///
/// ```rust
/// use anyhow::Result;
/// use deployment_changelog::api::rest::graphql::GraphQLClient;
/// use graphql_client::{GraphQLQuery, Response};
///
/// // Define a query using the graphql_client macro.
/// #[derive(GraphQLQuery)]
/// #[graphql(
///     schema_path = "path/to/schema.graphql",
///     query_path = "path/to/query.graphql",
///     response_derives = "Debug"
/// )]
/// struct MyQuery;
///
/// async fn execute_query() -> Result<Response<MyQuery::ResponseData>> {
///     // Create a new GraphQLClient instance.
///     let graphql_client = GraphQLClient::new("https://api.example.com")?;
///
///     // Set the query variables.
///     let variables = MyQuery::Variables { /* ... */ };
///
///     // Execute the query and return the result.
///     graphql_client.post(variables).await
/// }
/// ```
///
/// # Errors
///
/// Errors are handled using the `anyhow` crate, and the `Result` type is used to return errors
/// from functions. The `post` method can return errors related to HTTP requests, response
/// handling, or GraphQL-specific issues.
#[derive(Debug)]
pub struct GraphQLClient {
    client: RestClient
}

impl GraphQLClient {
    /// Creates a new `GraphQLClient` instance with the given base URL.
    ///
    /// The base URL is the root URL of the API server, without any path components.
    /// The client will append the GraphQL endpoint path to this base URL.
    ///
    /// # Example
    ///
    /// ```
    /// use deployment_changelog::api::rest::graphql::GraphQLClient;
    ///
    /// let graphql_client = GraphQLClient::new("https://api.example.com")?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL cannot be parsed or if there is an error
    /// creating the underlying `RestClient`.
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: RestClient::new(base_url)?
        })
    }

    /// Creates a new `GraphQLClient` instance using an existing `RestClient`.
    ///
    /// This method can be useful if you want to share a single `RestClient` instance
    /// between multiple clients, or if you need to customize the `RestClient` before
    /// passing it to the `GraphQLClient`.
    ///
    /// # Example
    ///
    /// ```
    /// use deployment_changelog::api::rest::{graphql::GraphQLClient, RestClient};
    ///
    /// let rest_client = RestClient::new("https://api.example.com")?;
    /// let graphql_client = GraphQLClient::from_client(rest_client);
    /// ```
    pub fn from_client(client: RestClient) -> Self {
        Self {
            client
        }
    }

    /// Executes a GraphQL query with the given variables and returns the response.
    ///
    /// The query is defined using the `GraphQLQuery` trait from the `graphql_client` crate.
    /// This method takes the variables required by the query and returns a `Response`
    /// containing the parsed response data or an error if the request fails.
    ///
    /// # Example
    ///
    /// ```
    /// use deployment_changelog::api::rest::graphql::GraphQLClient;
    /// use graphql_client::{GraphQLQuery, Response};
    ///
    /// #[derive(GraphQLQuery)]
    /// #[graphql(
    ///     schema_path = "path/to/schema.graphql",
    ///     query_path = "path/to/query.graphql",
    ///     response_derives = "Debug"
    /// )]
    /// struct MyQuery;
    ///
    /// async fn execute_query() -> Result<Response<MyQuery::ResponseData>> {
    ///     let graphql_client = GraphQLClient::new("https://api.example.com")?;
    ///     let variables = MyQuery::Variables { /* ... */ };
    ///     graphql_client.post(variables).await
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue with the HTTP request, response handling,
    /// or if the GraphQL API returns an error.
    pub async fn post<Q: GraphQLQuery>(&self, variables: Q::Variables) -> Result<Response<Q::ResponseData>> {
        let body = Q::build_query(variables);
        self.client.post_json::<Response<Q::ResponseData>, QueryBody<Q::Variables>>(GRAPHQL_ENDPOINT, &body)
            .await
            .with_context(|| {
                match serde_json::to_string_pretty(&body) {
                    Ok(body_serialized) => format!("Error making GraphQL call with query {0}", body_serialized),
                    Err(error) => format!("Error serializing GraphQL body: {error}")
                }
            })
    }
}

