use std::{time::Duration, collections::HashMap};

use reqwest::{Client, header::{HeaderMap, CONTENT_TYPE, HeaderValue, ACCEPT}, Url, Request, ClientBuilder};
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

    pub async fn execute<R: DeserializeOwned>(&self, request: Request) -> Result<R> {
        log::info!("Making request to {}", request.url());

        let response = self.client.execute(request).await
            .with_context(|| "Error executing request: {request}")?;

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
