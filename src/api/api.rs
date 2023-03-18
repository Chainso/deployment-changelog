use std::{time::Duration, collections::HashMap};

use reqwest::{Client, header::{HeaderMap, CONTENT_TYPE, HeaderValue, ACCEPT}, Error, Url, ClientBuilder, Request};
use serde::de::DeserializeOwned;

static APPLICATION_JSON: &str = "application/json";

pub struct RestClient {
    pub base_url: Url,
    pub client: Client,
}

impl RestClient {
    pub fn new(base_url: &str) -> Self {
        let mut headers: HeaderMap = HeaderMap::with_capacity(2);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON));
        headers.insert(ACCEPT, HeaderValue::from_static(APPLICATION_JSON));

        let url: Url = match Url::parse(base_url) {
            Ok(parsed_url) => parsed_url,
            Err(error) => panic!("Error parsing base URL {base_url}: {error}")
        };

        let client_builder: ClientBuilder = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(5));

        let client: Client = match client_builder.build() {
            Ok(cli) => cli,
            Err(error) => panic!("Error creating REST client with base URL {base_url}: {error}")
        };

        Self {
            base_url: url,
            client
        }
    }

    pub async fn get<R: DeserializeOwned>(&self, url: &str, query: Option<HashMap<&str, &str>>) -> R {
        let method = "GET";
        let request_url = self.build_url(url, method);

        let request = match self.client.get(request_url.clone()).query(&query).build() {
            Ok(req) => req,
            Err(error) => self.handle_request_build_error(&request_url, method, error)
        };

        self.execute(request).await
    }

    pub async fn execute<R: DeserializeOwned>(&self, request: Request) -> R {
        println!("Making request to {}", request.url());

        let response = match self.client.execute(request).await {
            Ok(res) => res,
            Err(error) => panic!("Error executing request: {error}")
        };

        match response.json::<R>().await {
            Ok(deserialized) => deserialized,
            Err(error) => panic!("Error deserializing response: {error}")
        }
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
