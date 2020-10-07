#[warn(clippy::all)]
use graphql_client::GraphQLQuery;
use std::time::Duration;

/// Implement by returning a parsed version of the epoch time stamp from RateLimit.
/// You may return a reasonable default or None if RateLimit isn't available.
pub trait BackoffTimer<R> {
    fn backoff(response: &R::ResponseData) -> Option<Duration>
    where
        R: GraphQLQuery + Send + Sync;
}
