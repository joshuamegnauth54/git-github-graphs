use crate::query_client::QueryClient;
use graphql_client::{GraphQLQuery, QueryBody, Response};
#[warn(clippy::all)]
use log::{error, info};

// The GitHub GraphQL schema defines types that don't necessarily map to Rust types.
// We'll need to define types such as URI ourselves as rustc throws
// an error originating from the macro otherwise.
// type URI = String;
type DateTime = String;

// Typing QueryBody<repo_view::Variables> gets old :(
type RepoQuery = QueryBody<repo_view::Variables>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/ghschema.graphql",
    query_path = "queries/repoquery.graphql",
    response_derives = "Clone,Debug"
)]
pub struct RepoView;

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

pub fn cursor(response: &repo_view::ResponseData) -> Option<String> {
    match response.repository {
        // Check if we have more pull requests via PageInfo
        Some(ref repo_data) if repo_data.pull_requests.page_info.has_next_page => repo_data
            .pull_requests
            .edges
            .as_ref()
            .and_then(|edges_vec| {
                edges_vec.iter().last().and_then(|vec_last| {
                    vec_last.as_ref().and_then(|edge| Some(edge.cursor.clone()))
                })
            }),
        _ => None,
    }
}

// Fix error handling later
pub async fn query_github(
    client: &QueryClient,
    repo_request: &RepoQuery,
) -> reqwest::Result<Response<repo_view::ResponseData>> {
    Ok(client.request::<RepoQuery, RepoView>(&repo_request).await?)
}

// Make this generic later.
pub async fn query_to_end(
    client: &QueryClient,
    init: &RepoQuery,
) -> reqwest::Result<Vec<repo_view::ResponseData>> {
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
            if let Some(cursor_s) = cursor(&data) {
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
