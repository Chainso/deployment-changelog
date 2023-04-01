use chrono::{DateTime, Local};
use graphql_client::GraphQLQuery;
use anyhow::{Result, Context, bail};

use super::graphql::GraphQLClient;

type InstantTime = DateTime<Local>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "resources/graphql/spinnaker/schema.graphql",
    query_path = "resources/graphql/spinnaker/queries.graphql",
    response_derives = "Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone"
)]
pub struct MdEnvironmentStatesQuery;

#[derive(Debug)]
pub struct SpinnakerClient {
    client: GraphQLClient
}

impl SpinnakerClient {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: GraphQLClient::new(base_url)?
        })
    }

    pub fn from_client(client: GraphQLClient) -> Self {
        Self {
            client
        }
    }

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
