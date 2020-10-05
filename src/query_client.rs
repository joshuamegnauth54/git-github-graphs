#[warn(clippy::all)]
use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client, Result};
use serde::Serialize;

const GITHUBAPI: &str = "https://api.github.com/graphql";
// User agents are always required for the GitHub API.
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), " (", env!("CARGO_PKG_VERSION"), ")");
// GITHUB_API_TOKEN is the standard environmental variable for the token.
const TOKEN_ENV: &str = "GITHUB_API_TOKEN";

pub struct QueryClient {
    client: Client,
    token: String,
}

impl QueryClient {
    pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(QueryClient {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .gzip(true)
                .build()?,
            token: std::env::var(TOKEN_ENV)?,
        })
    }

    pub async fn request<Q, R>(&self, query: &Q) -> Result<Response<R::ResponseData>>
    where
        Q: Serialize,
        R: GraphQLQuery + Send + Sync,
    {
        // The block below simply sends a POST to the GitHub GraphQL API site with the provided
        // token and query.
        Ok(self
            .client
            .post(GITHUBAPI)
            .bearer_auth(&self.token)
            .json(&query)
            .send()
            .await?
            .json()
            .await?)
    }

    fn backoff(&self) {
        unimplemented!();
    }
}
