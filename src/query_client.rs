#[warn(clippy::all)]
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use serde::Serialize;
use std::{thread::sleep, time::Duration};

use crate::{
    error::{Error, ErrorKind, Result},
    query_structs::backoff_timer::BackoffTimer,
};

const DEFAULT_TIMEOUT: u64 = 10;
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
    pub fn new() -> Result<Self> {
        Ok(QueryClient {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .gzip(true)
                .build()?,
            token: std::env::var(TOKEN_ENV)
                .map_err(|_e| Error::new("Creating reqwest::Client.", ErrorKind::NoToken))?,
        })
    }

    pub async fn request<Q, R>(&self, query: &Q) -> Result<Response<R::ResponseData>>
    where
        Q: Serialize,
        R: BackoffTimer<R> + GraphQLQuery + Send + Sync + Unpin,
    {
        // The block below queries the GitHub API using the associated token and query. I'm saving
        // the result into a variable to query R::backoff().
        let result: Result<Response<R::ResponseData>> = self
            .client
            .post(GITHUBAPI)
            .bearer_auth(&self.token)
            .json(&query)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| {
                Error::new(
                    "Deserializing JSON into ResponseData",
                    ErrorKind::Reqwest(e),
                )
            });

        QueryClient::backoff::<R>(&result).await;
        result
    }

    // The backoff function defers to R::backoff for the timer. If the implementer does not use the
    // rate limit info but returns some other amount of time we still defer to their wisdom.
    // Likewise, if the implementer returns None we simply use a default.
    async fn backoff<R>(response: &Result<Response<R::ResponseData>>)
    where
        R: BackoffTimer<R> + GraphQLQuery + Send + Sync + Unpin,
    {
        match response {
            Ok(ref repdata) if repdata.data.is_some() => sleep(
                R::backoff(&repdata.data.as_ref().unwrap())
                    .unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT)),
            ),
            _ => sleep(Duration::from_secs(DEFAULT_TIMEOUT)),
        }
    }
}
