#[warn(clippy::all)]
use graphql_client::{GraphQLQuery, QueryBody};

// The GitHub GraphQL schema defines types that don't necessarily map to Rust types.
// We'll need to define types such as URI ourselves as rustc throws
// an error originating from the macro otherwise.
// type URI = String;
type DateTime = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/ghschema.graphql",
    query_path = "queries/repoquery.graphql",
    response_derives = "Debug"
)]
pub struct RepoView;

pub fn repoview_request<V: AsRef<str>>(
    owner: V,
    name: V,
    nnodes: i64,
    pullcursor: Option<String>,
) -> QueryBody<repo_view::Variables> {
    RepoView::build_query(repo_view::Variables {
        owner: owner.as_ref().to_owned(),
        name: name.as_ref().to_owned(),
        nnodes,
        pullcursor,
    })
}
