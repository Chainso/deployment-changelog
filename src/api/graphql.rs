use anyhow::{Context, Result};
use graphql_client::{GraphQLQuery, QueryBody, Response};

use super::rest::RestClient;

const GRAPHQL_ENDPOINT: &str = "graphql";

#[derive(Debug)]
pub struct GraphQLClient {
    client: RestClient
}

impl GraphQLClient {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: RestClient::new(base_url)?
        })
    }

    pub fn from_client(client: RestClient) -> Self {
        Self {
            client
        }
    }

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

