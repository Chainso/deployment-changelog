use std::{time::Duration, collections::HashMap};

use reqwest::{Client, header::{HeaderMap, CONTENT_TYPE, HeaderValue, ACCEPT}, Error, Url, Request};
use serde::de::DeserializeOwned;
use anyhow::{Context, Result};

static APPLICATION_JSON: &str = "application/json";

#[async_trait::async_trait]
pub trait Paginated<T: Send> {
    async fn next(&mut self) -> Result<Vec<T>>;
    fn is_last(&self) -> bool;

    async fn all(&mut self) -> Result<Vec<T>> {
        let mut all_results = Vec::new();

        while !self.is_last() {
            all_results.extend(self.next().await?);
        }

        Ok(all_results)
    }
}

#[derive(Debug)]
pub struct RestClient {
    pub base_url: Url,
    pub client: Client,
}

impl RestClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let mut headers: HeaderMap = HeaderMap::with_capacity(2);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON));
        headers.insert(ACCEPT, HeaderValue::from_static(APPLICATION_JSON));

        let url = Url::parse(base_url)
            .with_context(|| format!("Error parsing base URL {base_url}"))?;

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(5))
            .build()
            .with_context(|| format!("Error creating REST client with base URL {base_url}"))?;

        Ok(Self {
            base_url: url,
            client
        })
    }

    pub async fn get<R: DeserializeOwned>(&self, url: &str, query: Option<&HashMap<String, String>>) -> Result<R> {
        let method = "GET";
        let request_url = self.build_url(url, method);

        let request = match self.client.get(request_url.clone()).query(&query).build() {
            Ok(req) => req,
            Err(error) => self.handle_request_build_error(&request_url, method, error)
        };

        self.execute(request).await
    }

    pub async fn execute<R: DeserializeOwned>(&self, request: Request) -> Result<R> {
        log::info!("Making request to {}", request.url());

        let response = self.client.execute(request).await
            .with_context(|| "Error executing request: {request}")?;

        return response.json::<R>().await
            .with_context(|| "Error deserializing response");
    }

    pub fn build_url(&self, url: &str, method: &str) -> Url {
        match self.base_url.join(url) {
            Ok(full_url) => full_url,
            Err(error) => panic!("Error creating {method} request URL with base URL {0} and path {url}: {error}", self.base_url)
        }
    }

    fn handle_request_build_error(&self, request_url: &Url, method: &str, error: Error) -> Request {
        panic!("Error building {method} request for url {request_url}: {error}");
    }
}
