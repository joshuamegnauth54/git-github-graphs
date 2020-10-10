#[warn(clippy::all)]
use futures::future::select_all;
use graphql_client::QueryBody;
use log::info;
use std::env::args;

mod error;
mod errorkind;
mod query_client;
mod query_structs;
use error::{Error, Result};
use errorkind::ErrorKind;
use query_client::QueryClient;
use query_structs::{repoview::*, repoview_nodes::RepoViewNode};

// I set NUM_NODES to a reasonable default rather than taking arguments. The API throws an error if
// the caller may possibly request more than 500,000 nodes at a time. I would either have to
// rewrite the query to take more variables, manually check and throw an error, or let the API
// return errors to the user (which is probably just myself to be honest).
const NUM_NODES: i64 = 50;

// Figure out lifetimes later instead of creating Strings
struct RepositoryArg {
    owner: String,
    name: String,
}

// Parses command line arguments. This program only takes repository names.
fn parse_args() -> Result<Vec<RepositoryArg>> {
    if args().len() < 2 {
        Err(Error::new("No arguments found", ErrorKind::BadArgs))
    } else {
        args()
            .skip(1)
            .map(|arg| {
                let mut repo = arg.split('/');
                Ok(RepositoryArg {
                    owner: repo
                        .next()
                        .ok_or_else(|| {
                            Error::new(format!("Parsing repository ({})", arg), ErrorKind::BadArgs)
                        })?
                        .to_owned(),
                    name: repo
                        .next()
                        .ok_or_else(|| {
                            Error::new(format!("Parsing repository ({})", arg), ErrorKind::BadArgs)
                        })?
                        .to_owned(),
                })
            })
            .collect()
    }
}

// Convenience function to make a Vector of requests.
fn make_requests() -> Result<Vec<QueryBody<repo_view::Variables>>> {
    Ok(parse_args()?
        .into_iter()
        .map(|repository| repoview_request(repository.owner, repository.name, NUM_NODES, None))
        .collect())
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _log =
        pretty_env_logger::try_init().map_err(|e| eprintln!("Failed to initialize logger: {}", e));

    let requests = make_requests()?;
    let client = QueryClient::new()?;
    info!("Beginning scrape.");
    // Test
    let response = query_to_end(&client, requests.iter().next().unwrap()).await?;
    println!("{:#?}", response);

    let nodes_test = RepoViewNode::parse_nodes(&response);
    println!("{:#?}", nodes_test);

    Ok(())
}
