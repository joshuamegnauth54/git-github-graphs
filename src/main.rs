#[warn(clippy::all)]
use log::info;

mod query_client;
mod query_structs;
use query_client::QueryClient;
use query_structs::repoview::*;

use query_structs::repoview_nodes::RepoViewNode;

// Will handle args later.
const NUM_NODES: i64 = 50;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _log =
        pretty_env_logger::try_init().map_err(|e| eprintln!("Failed to initialize logger: {}", e));

    let rustlang_req = repoview_request("bevyengine", "bevy", NUM_NODES, None);
    let client = QueryClient::new()?;
    info!("Beginning scrape.");
    let response = query_to_end(&client, &rustlang_req).await?;
    //println!("{:?}", response);
    let fuck = RepoViewNode::parse_nodes(&response);
    Ok(())
}
