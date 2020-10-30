#![feature(backtrace)]
use futures::future::join_all;
use graphql_client::QueryBody;
use log::{error, info};
use std::{
    env::args,
    fs::{create_dir_all, File},
    path::Path,
};

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

fn write_output(results: &[Vec<RepoViewNode>]) {
    // Open a set of output files with the paths output/owner/repo.json.
    // We'll attempt to write the data regardless of any errors rather than simply failing.
    let files: Vec<Result<File>> = results
        .iter()
        .map(|nodes| {
            nodes
                .iter()
                // Peek to look at the first node to get the repository name.
                .peekable()
                .peek()
                // Report if the Vector is empty then continue.
                .ok_or_else(|| {
                    Error::new(
                        "Empty input data while writing output JSON.",
                        ErrorKind::EmptyData,
                    )
                })
                .and_then(|node| {
                    // Paths are a zero cost conversion so we need a variable.
                    let temp_path = format!("output/{}.json", &node.repository);
                    let repo_path = Path::new(&temp_path);
                    // Create the full directory path if required or return an error with the
                    // failed path.
                    create_dir_all(&repo_path.parent().ok_or_else(|| {
                        // Manually convert NoneError into an Error.
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            repo_path.to_str().unwrap_or_else(|| "").to_owned(),
                        )
                    })?)?;
                    Ok(File::create(&repo_path)?)
                })
        })
        .collect();

    // I'm not sure what else to do beyond reporting the errors.
    for (file_opt, nodes) in files.iter().zip(results.iter()) {
        match file_opt {
            Ok(file) => {
                if let Err(e) = write_nodes(file, &nodes) {
                    error!("{}", e)
                }
            }
            Err(e) => error!("{}", e),
        }
    }
}

/// Convenience function to run all queries then return the results.
async fn query_all(
    client: &QueryClient,
    queries: &[QueryBody<repo_view::Variables>],
) -> Vec<Result<Vec<repo_view::ResponseData>>> {
    let futures: Vec<_> = queries
        .iter()
        .map(|query| query_to_end(&client, &query))
        .collect();
    join_all(futures).await
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _log =
        pretty_env_logger::try_init().map_err(|e| eprintln!("Failed to initialize logger: {}", e));

    let requests = make_requests()?;
    let client = QueryClient::new()?;
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
    // I think I flattened everything too much?
    write_output(&vec![parsed_data]);

    Ok(())
}
