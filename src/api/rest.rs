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

use reqwest::{Client, header::{HeaderMap, CONTENT_TYPE, HeaderValue, ACCEPT}, Url, Request, ClientBuilder};
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
    /// Fetches the next page of results and returns a vector of instances of the generic type T.
    ///
    /// # Example
    ///
    /// ```
    /// let mut paginated_commits = bitbucket_client.compare_commits("PROJECT", "REPO", "start_commit", "end_commit");
    /// let next_page_commits = paginated_commits.next().await?;
    /// ```
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of instances of the generic type T or an error if the request fails.
    async fn next(&mut self) -> Result<Vec<T>>;

    /// Determines whether the last page of results has been reached.
    ///
    /// # Example
    ///
    /// ```
    /// let mut paginated_commits = bitbucket_client.compare_commits("PROJECT", "REPO", "start_commit", "end_commit");
    /// let is_last_page = paginated_commits.is_last();
    /// ```
    ///
    /// # Returns
    ///
    /// A boolean value indicating whether the last page of results has been reached.
    fn is_last(&self) -> bool;

    /// Fetches all pages of results and returns a vector of instances of the generic type T.
    ///
    /// This method repeatedly calls `next()` until `is_last()` returns true.
    ///
    /// # Example
    ///
    /// ```
    /// let mut paginated_commits = bitbucket_client.compare_commits("PROJECT", "REPO", "start_commit", "end_commit");
    /// let all_commits = paginated_commits.all().await?;
    /// ```
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of instances of the generic type T or an error if the request fails.
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
    /// Creates a new `RestClient` instance with the given base URL.
    ///
    /// # Example
    ///
    /// ```
    /// let rest_client = RestClient::new("https://api.bitbucket.org").unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the REST API to be accessed.
    ///
    /// # Returns
    ///
    /// A Result containing a new RestClient instance or an error if the base URL cannot be parsed.
    pub fn new(base_url: &str) -> Result<Self> {
        RestClient::builder(base_url)?
            .build()
    }

    /// Creates a new `RestClientBuilder` instance with the given base URL.
    ///
    /// # Example
    ///
    /// ```
    /// let rest_client_builder = RestClient::builder("https://api.bitbucket.org").unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the REST API to be accessed.
    ///
    /// # Returns
    ///
    /// A Result containing a new RestClientBuilder instance or an error if the base URL cannot be parsed.
    pub fn builder(base_url: &str) -> Result<RestClientBuilder> {
        RestClientBuilder::new(base_url)
    }

    /// Sends a GET request to the specified URL and deserializes the response to the generic type R.
    ///
    /// # Example
    ///
    /// ```
    /// let commits: Vec<Commit> = rest_client.get("https://api.bitbucket.org/api/rest/2.0/repositories/user/repo/commits", None).await.unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the resource to be fetched.
    /// * `query` - An optional HashMap of query parameters to be included in the request.
    ///
    /// # Returns
    ///
    /// A Result containing an instance of the generic type R or an error if the request fails.
    pub async fn get<R: DeserializeOwned>(&self, url: &str, query: Option<&HashMap<String, String>>) -> Result<R> {
        let method = "GET";
        let request_url = self.build_url(url, method)?;

        let request = self.client.get(request_url.clone())
            .query(&query)
            .build()?;

        self.execute(request).await
    }

    /// Sends a POST request to the specified URL with a JSON body and deserializes the response to the generic type R.
    ///
    /// # Example
    ///
    /// ```
    /// let new_comment = NewComment { content: "This is a comment.".to_string() };
    /// let comment: Comment = rest_client.post_json("https://api.bitbucket.org/api/rest/2.0/repositories/user/repo/pullrequests/1/comments", &new_comment).await.unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the resource to be created or updated.
    /// * `json_body` - The JSON body to be sent with the request.
    ///
    /// # Returns
    ///
    /// A Result containing an instance of the generic type R or an error if the request fails.
    pub async fn post_json<R: DeserializeOwned, J: Serialize + ?Sized>(&self, url: &str, json_body: &J) -> Result<R> {
        let method = "POST";
        let request_url = self.build_url(url, method)?;

        let request = self.client.post(request_url.clone())
            .json(json_body)
            .build()?;

        self.execute(request).await
    }

    /// Executes the given `Request` and deserializes the response to the generic type R.
    ///
    /// # Example
    ///
    /// ```
    /// let request = rest_client.client.get("https://api.bitbucket.org/api/rest/2.0/repositories/user/repo/commits")
    ///     .build()
    ///     .unwrap();
    /// let commits: Vec<Commit> = rest_client.execute(request).await.unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `request` - The `Request` to be executed.
    ///
    /// # Returns
    ///
    /// A Result containing an instance of the generic type R or an error if the request fails.
    pub async fn execute<R: DeserializeOwned>(&self, request: Request) -> Result<R> {
        log::info!("Making request to {}", request.url());

        let response = self.client.execute(request).await
            .with_context(|| "Error executing request")?;

        return response.json::<R>().await
            .with_context(|| "Error deserializing response");
    }

    /// Constructs a `Url` using the base URL and the provided path.
    ///
    /// # Example
    ///
    /// ```
    /// let url = rest_client.build_url("/2.0/repositories/user/repo/commits", "GET").unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `url` - The path to be appended to the base URL.
    /// * `method` - The HTTP method (e.g., "GET", "POST") for which the URL is being constructed.
    ///
    /// # Returns
    ///
    /// A Result containing the constructed `Url` or an error if the URL cannot be created.
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
    /// Creates a new instance of `RestClientBuilder` with the given base URL.
    ///
    /// The builder has default headers and a timeout of 5 seconds.
    ///
    /// # Example
    ///
    /// ```
    /// let rest_client_builder = RestClientBuilder::new("https://api.bitbucket.org").unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL for the REST API.
    ///
    /// # Returns
    ///
    /// A Result containing an instance of `RestClientBuilder` or an error if the URL cannot be parsed.
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
    
    /// Constructs a `RestClient` using the settings from the `RestClientBuilder`.
    ///
    /// # Example
    ///
    /// ```
    /// let rest_client_builder = RestClientBuilder::new("https://api.bitbucket.org").unwrap();
    /// let rest_client = rest_client_builder.build().unwrap();
    /// ```
    ///
    /// # Returns
    ///
    /// A Result containing an instance of `RestClient` or an error if the client cannot be created.
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
