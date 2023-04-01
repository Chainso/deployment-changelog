//! This module provides functionality to interact with REST APIs
//! using a simple and ergonomic interface. It offers a `RestClient` and `RestClientBuilder` for
//! making requests and handling pagination, as well as a `Paginated` trait for pagination support.
//!
//! # Examples
//!
//! Creating a new `RestClient` with a base URL:
//!
//! ```rust
//! use deployment_changelog::api::rest::RestClient;
//!
//! let rest_client = RestClient::new("https://api.example.com").unwrap();
//! ```
//!
//! Using the `RestClient` to make a GET request:
//!
//! ```rust
//! use deployment_changelog::api::rest::RestClient;
//! use serde::Deserialize;
//! use std::collections::HashMap;
//!
//! #[derive(Deserialize, Debug)]
//! struct ResponseData {
//!     key: String,
//!     value: String,
//! }
//!
//! let rest_client = RestClient::new("https://api.example.com").unwrap();
//!
//! let query_params = {
//!     let mut map = HashMap::new();
//!     map.insert("key".to_string(), "value".to_string());
//!     map
//! };
//!
//! let response: ResponseData = rest_client.get("/endpoint", Some(&query_params)).await.unwrap();
//! println!("{:?}", response);
//! ```
//!
//! Implementing the `Paginated` trait for a custom type:
//!
//! ```rust
//! use deployment_changelog::api::rest::{RestClient, Paginated};
//! use serde::Deserialize;
//! use anyhow::Result;
//!
//! #[derive(Deserialize, Debug)]
//! struct ResponseData {
//!     items: Vec<String>,
//!     has_more: bool,
//! }
//!
//! struct PaginatedItems {
//!     rest_client: RestClient,
//!     endpoint: String,
//!     has_more: bool,
//! }
//!
//! #[async_trait::async_trait]
//! impl Paginated<String> for PaginatedItems {
//!     async fn next(&mut self) -> Result<Vec<String>> {
//!         let response: ResponseData = self.rest_client.get(&self.endpoint, None).await?;
//!         self.has_more = response.has_more;
//!         Ok(response.items)
//!     }
//!
//!     fn is_last(&self) -> bool {
//!         !self.has_more
//!     }
//! }
//! ```
//!
//! Using the `all()` method to fetch all paginated results:
//!
//! ```rust
//! use deployment_changelog::api::rest::{RestClient, Paginated};
//! use serde::Deserialize;
//! use anyhow::Result;
//!
//! // ... (PaginatedItems definition as in the previous example)
//!
//! let rest_client = RestClient::new("https://api.example.com").unwrap();
//! let mut paginated_items = PaginatedItems {
//!     rest_client,
//!     endpoint: "/endpoint".to_string(),
//!     has_more: true,
//! };
//!
//! let all_items = paginated_items.all().await.unwrap();
//! println!("{:?}", all_items);
//! ```
//!
//! This module aims to provide an easy-to-use interface for interacting with REST APIs,
//! handling pagination and deserialization of the responses.
use std::{time::Duration, collections::HashMap};

use reqwest::{Client, header::{HeaderMap, CONTENT_TYPE, HeaderValue, ACCEPT}, Url, Request, ClientBuilder, Body};
use serde::{de::DeserializeOwned, Serialize};
use anyhow::{Context, Result};

static APPLICATION_JSON: &str = "application/json";

/// The `Paginated` trait provides an interface for handling pagination in REST APIs. It offers
/// methods for retrieving the next set of results and checking if there are more results available.
/// Additionally, it provides a convenient `all()` method to fetch all results across multiple pages.
///
/// # Examples
///
/// Implementing the `Paginated` trait for a custom type:
///
/// ```rust
/// use deployment_changelog::api::rest::{RestClient, Paginated};
/// use serde::Deserialize;
/// use anyhow::Result;
///
/// #[derive(Deserialize, Debug)]
/// struct ResponseData {
///     items: Vec<String>,
///     has_more: bool,
/// }
///
/// struct PaginatedItems {
///     rest_client: RestClient,
///     endpoint: String,
///     has_more: bool,
/// }
///
/// #[async_trait::async_trait]
/// impl Paginated<String> for PaginatedItems {
///     async fn next(&mut self) -> Result<Vec<String>> {
///         let response: ResponseData = self.rest_client.get(&self.endpoint, None).await?;
///         self.has_more = response.has_more;
///         Ok(response.items)
///     }
///
///     fn is_last(&self) -> bool {
///         !self.has_more
///     }
/// }
/// ```
///
/// Using the `all()` method to fetch all paginated results:
///
/// ```rust
/// use deployment_changelog::api::rest::{RestClient, Paginated};
/// use serde::Deserialize;
/// use anyhow::Result;
///
/// // ... (PaginatedItems definition as in the previous example)
///
/// let rest_client = RestClient::new("https://api.example.com").unwrap();
/// let mut paginated_items = PaginatedItems {
///     rest_client,
///     endpoint: "/endpoint".to_string(),
///     has_more: true,
/// };
///
/// let all_items = paginated_items.all().await.unwrap();
/// println!("{:?}", all_items);
/// ```
#[async_trait::async_trait]
pub trait Paginated<T: Send> {
    /// Retrieve the next set of results from the API.
    async fn next(&mut self) -> Result<Vec<T>>;

    /// Check if the current page is the last page.
    fn is_last(&self) -> bool;

    /// Retrieve all results from the API.
    async fn all(&mut self) -> Result<Vec<T>> {
        let mut all_results = Vec::new();

        // Keep retrieving results until the last page is reached.
        while !self.is_last() {
            all_results.extend(self.next().await?);
        }

        Ok(all_results)
    }
}

/// The `RestClient` struct is responsible for making HTTP requests to REST APIs.
/// It provides an easy-to-use interface for making requests, handling response deserialization,
/// and working with pagination.
///
/// # Examples
///
/// Creating a new `RestClient` with a base URL:
///
/// ```rust
/// use deployment_changelog::api::rest::RestClient;
///
/// let rest_client = RestClient::new("https://api.example.com").unwrap();
/// ```
///
/// Using the `RestClient` to make a GET request:
///
/// ```rust
/// use deployment_changelog::api::rest::RestClient;
/// use serde::Deserialize;
/// use std::collections::HashMap;
///
/// #[derive(Deserialize, Debug)]
/// struct ResponseData {
///     key: String,
///     value: String,
/// }
///
/// let rest_client = RestClient::new("https://api.example.com").unwrap();
///
/// let query_params = {
///     let mut map = HashMap::new();
///     map.insert("key".to_string(), "value".to_string());
///     map
/// };
///
/// let response: ResponseData = rest_client.get("/endpoint", Some(&query_params)).await.unwrap();
/// println!("{:?}", response);
/// ```
#[derive(Debug)]
pub struct RestClient {
    pub base_url: Url,
    pub client: Client,
}

impl RestClient {
    pub fn new(base_url: &str) -> Result<Self> {
        RestClient::builder(base_url)?
            .build()
    }

    pub fn builder(base_url: &str) -> Result<RestClientBuilder> {
        RestClientBuilder::new(base_url)
    }

    pub async fn get<R: DeserializeOwned>(&self, url: &str, query: Option<&HashMap<String, String>>) -> Result<R> {
        let method = "GET";
        let request_url = self.build_url(url, method)?;

        let request = self.client.get(request_url.clone())
            .query(&query)
            .build()?;

        self.execute(request).await
    }

    pub async fn post_json<R: DeserializeOwned, J: Serialize + ?Sized>(&self, url: &str, json_body: &J) -> Result<R> {
        let method = "POST";
        let request_url = self.build_url(url, method)?;

        let request = self.client.post(request_url.clone())
            .json(json_body)
            .build()?;

        self.execute(request).await
    }

    pub async fn execute<R: DeserializeOwned>(&self, request: Request) -> Result<R> {
        log::info!("Making request to {}", request.url());

        let response = self.client.execute(request).await
            .with_context(|| "Error executing request")?;

        return response.json::<R>().await
            .with_context(|| "Error deserializing response");
    }

    pub fn build_url(&self, url: &str, method: &str) -> Result<Url> {
        self.base_url.join(url) 
            .with_context(|| {
                format!(
                    "Error creating {method} request URL with base URL {0} and path {url}",
                    self.base_url
                )
        })
    }
}

/// The `RestClientBuilder` struct provides a convenient way to customize and build a `RestClient`
/// instance, setting sane defaults.
///
/// # Examples
///
/// Creating a new `RestClient` with a base URL and custom timeout using `RestClientBuilder`:
///
/// ```rust
/// use deployment_changelog::api::rest::RestClientBuilder;
/// use std::time::Duration;
///
/// let rest_client_builder = RestClientBuilder::new("https://api.example.com")
///     .unwrap();
///
/// rest_client_builder.client_builder = rest_client_builder.client_builder
///     .timeout(Duration::from_secs(10));
///
/// let rest_client = rest_client_builder.build()
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct RestClientBuilder {
    pub base_url: Url,
    pub client_builder: ClientBuilder
}

impl RestClientBuilder {
    pub fn new(base_url: &str) -> Result<Self> {
        let mut headers: HeaderMap = HeaderMap::with_capacity(2);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON));
        headers.insert(ACCEPT, HeaderValue::from_static(APPLICATION_JSON));

        let url = Url::parse(base_url)
            .with_context(|| format!("Error parsing base URL {base_url}"))?;

        let client_builder = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(5));

        Ok(Self {
            base_url: url,
            client_builder
        })
    }
    
    pub fn build(self) -> Result<RestClient> {
        let client = self.client_builder
            .build()
            .with_context(|| format!("Error creating REST client with base URL {0}", self.base_url))?;

        Ok(RestClient {
            base_url: self.base_url,
            client
        })
    }
}
