use super::{backoff_timer::BackoffTimer, cursor::Cursor};
// Importing error::Result breaks #[derive(GraphQLQuery)] for some reason.
use crate::{error::Result as GGGResult, query_client::QueryClient};
use chrono::{offset::Utc, Duration as OldDuration};
use graphql_client::{GraphQLQuery, QueryBody, Response};
use log::{error, info, warn};
use std::time::Duration;

// The GitHub GraphQL schema defines types that don't necessarily map to Rust types.
// We'll need to define types such as URI ourselves as rustc throws an error originating from the macro otherwise.
// type URI = String;
type DateTime = String;

// Typing QueryBody<repo_view::Variables> gets old :(
type RepoQuery = QueryBody<repo_view::Variables>;

// Defaults
// Minutes is signed and seconds is unsigned due to the type constraints for the two different
// durations. I don't see a reason to make them the same type and convert.
const SLEEP_MINUTES: i64 = 15;
const SLEEP_SEC: u64 = 900;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/ghschema.graphql",
    query_path = "queries/repoquery.graphql",
    response_derives = "Clone,Debug"
)]
pub struct RepoView;

impl Cursor<RepoView> for RepoView {
    fn cursor(response: &repo_view::ResponseData) -> Option<String> {
        match response.repository {
            // Check if we have more pull requests via PageInfo
            Some(ref repo_data) if repo_data.pull_requests.page_info.has_next_page => repo_data
                .pull_requests
                .edges
                .as_ref()
                .and_then(|edges_vec| {
                    edges_vec
                        .iter()
                        .last()
                        .and_then(|vec_last| vec_last.as_ref().map(|edge| edge.cursor.clone()))
                }),
            _ => None,
        }
    }
}

impl BackoffTimer<RepoView> for RepoView {
    fn backoff(response: &repo_view::ResponseData) -> Option<Duration> {
        match &response.rate_limit {
            Some(ratelimit) if ratelimit.remaining == 0 => {
                // The following is a bit messy but hopefully simple to follow.
                // I don't want to return any errors because handling them in a higher context
                // would be messy when waiting for a default time would be easier. However, parsing
                // the rate limit DateTime String or subtracting from Utc::now() shouldn't really
                // fail so printing a message seems like a good warning.
                let reset_at = chrono::DateTime::parse_from_rfc3339(&ratelimit.reset_at)
                    .unwrap_or_else(|e| {
                        warn!(
                            "Error parsing an ostensibly existing RateLimit DateTime: {}",
                            e
                        );
                        // Notify the user if we can't parse the rate_limit DateTime followed by
                        // returning a default sleep time.
                        (Utc::now() + OldDuration::minutes(SLEEP_MINUTES)).into()
                    });
                info!("Rate limit reached. Sleeping until: {}", reset_at);
                // Same process as above. Check if the conversion is okay or return the default.
                Some(
                    (chrono::DateTime::<Utc>::from(reset_at) - Utc::now())
                        .to_std()
                        .unwrap_or_else(|e| {
                            warn!("{}", e);
                            Duration::from_secs(SLEEP_SEC)
                        }),
                )
            }
            Some(ratelimit) => {
                info!("Queries remaining before pausing: {}", ratelimit.remaining);
                None
            }
            _ => None,
        }
    }
}

/// Convenience function to build a query.
pub fn repoview_request<V: AsRef<str>>(
    owner: V,
    name: V,
    nnodes: i64,
    pullcursor: Option<String>,
) -> RepoQuery {
    RepoView::build_query(repo_view::Variables {
        owner: owner.as_ref().to_owned(),
        name: name.as_ref().to_owned(),
        nnodes,
        pullcursor,
    })
}

/// Executes the GraphQL Query defined in queries/repoquery.graphql on the repository and variables defined in
/// repo_request.
/// Note that the query is only executed once.
pub async fn query_github(
    client: &QueryClient,
    repo_request: &RepoQuery,
) -> GGGResult<Response<repo_view::ResponseData>> {
    Ok(client.request::<RepoQuery, RepoView>(&repo_request).await?)
}

// Make this generic later?
/// Fully gathers the data requested by queries/repoquery.graphql on the repository defined in init
/// until all data is gathered. The $cursor variable is automatically updated (i.e. paginated).
pub async fn query_to_end(
    client: &QueryClient,
    init: &RepoQuery,
) -> GGGResult<Vec<repo_view::ResponseData>> {
    // Holds raw responses to process elsewhere.
    let mut responses: Vec<repo_view::ResponseData> = Vec::new();

    // Cloning the query itself didn't seem to work as the object remained "behind an immutable
    // reference" for whatever reason. I assume the Clone trait isn't derived on
    // GraphQLQuery::Variables or whatever? Either way, manually cloning for now.
    let mut query = repoview_request(
        init.variables.owner.clone(),
        init.variables.name.clone(),
        init.variables.nnodes,
        init.variables.pullcursor.clone(),
    );
    info!(
        "Scraping from {}/{}",
        query.variables.owner, query.variables.name
    );
    // Handle this better later...must submit assignment.
    loop {
        let last_resp = query_github(&client, &query).await?;
        if let Some(data) = last_resp.data {
            // No cursor = no more data
            if let Some(cursor_s) = RepoView::cursor(&data) {
                // The old cursor must be replaced with the new, latest cursor in order to
                // paginate.
                query.variables.pullcursor = Some(cursor_s);
                responses.push(data);
            } else {
                responses.push(data);
                break;
            }
        }

        // The errors field is an Option that may coexist with the data field. In other words, we
        // may receive data _and_ have errors. I don't know if parsing the errors to figure out if
        // we should break is worth the effort. Likely, we'll quit due to the cursor if the error
        // is extreme or may continue if the errors aren't too bad. So, let's just print the errors
        // here.
        if let Some(errors) = last_resp.errors {
            for e in errors.iter() {
                error! {"GraphQL error: {}", e};
            }
        }
    }

    Ok(responses)
}
