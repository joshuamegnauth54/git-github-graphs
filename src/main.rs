#![feature(backtrace)]
use graphql_client::QueryBody;
use log::{error, info};
use std::{env::args, fs::File, path::Path};

mod error;
mod errorkind;
mod query_client;
mod query_structs;
use error::{Error, Result};
use errorkind::ErrorKind;
use query_client::QueryClient;
use query_structs::{repoview::*, repoview_nodes::RepoViewNode, write_nodes::write_nodes};

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

fn write_output<P: AsRef<Path>>(results: &[RepoViewNode], path: P) -> Result<()> {
    let file = File::create(&path)?;
    Ok(write_nodes(file, &results)?)
}

/// Convenience function to run all queries then return the results.
async fn query_all(
    client: &QueryClient,
    queries: &[QueryBody<repo_view::Variables>],
) -> Vec<Result<Vec<repo_view::ResponseData>>> {
    let mut results: Vec<_> = Vec::new();
    // I have to run these synchronously or else GitHub yells at me.
    for query in queries.iter() {
        results.push(query_to_end(&client, &query).await);
    }

    results
}

#[tokio::main]
async fn main() -> Result<()> {
    let _log =
        pretty_env_logger::try_init().map_err(|e| eprintln!("Failed to initialize logger: {}", e));

    let requests = match make_requests() {
        Ok(req) => req,
        Err(e) /*if e.errorkind == ErrorKind::BadArgs =>*/ => {
            error!("You must provide repositories in the form owner/repo (ex. joshuamegnauth54/cat_tracker).");
            return Err(e)
        }
    };

    let client = match QueryClient::new() {
        Ok(qc) => qc,
        Err(e) /*if e.errorkind == ErrorKind::NoToken*/ => {
            error!("Error! Do you have your token in the environmental variable GITHUB_API_TOKEN?");
            return Err(e)
        }

    };

    info!("Beginning scrape.");
    let (responses_nested, errors) = query_all(&client, &requests)
        .await
        .into_iter()
        .partition::<Vec<_>, _>(Result::is_ok);

    for error in errors {
        // Partitioned into ok/err so we can unwrap the error here.
        error!("Error returned during query phase: {}", error.unwrap_err());
    }

    info!("Parsing nodes.");
    let responses: Vec<_> = responses_nested.into_iter().flatten().flatten().collect();
    //info!("Size: {}", responses.len());
    let parsed_data = RepoViewNode::parse_nodes(&responses);
    info!("Writing files.");
    write_output(&parsed_data, "output/output.json")?;

    Ok(())
}
