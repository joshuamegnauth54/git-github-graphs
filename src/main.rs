#[warn(clippy::all)]
use log::info;

mod query_client;
mod query_structs;
use query_client::QueryClient;
use query_structs::repoview::*;

// Will handle args later.
const NUM_NODES: i64 = 100;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _log =
        pretty_env_logger::try_init().map_err(|e| eprintln!("Failed to initialize logger: {}", e));

    let rustlang_req = repoview_request("rust-lang", "rust", 5, None);
    let client = QueryClient::new()?; //expect("Client failed. TLS?");
                                      /*let response = client
                                          .request::<QueryBody<repo_view::Variables>, RepoView>(&rustlang_req)
                                          .await?;

                                      if let Some(ref data) = response.data {
                                          println!("{:#?}", data);
                                      }

                                      if let Some(errors) = response.errors {
                                          println!("{:#?}", errors);
                                      }

                                      println!("Cursor: {:?}", cursor(&response.data.unwrap()));*/

    info!("Beginning scrape.");
    let response = query_to_end(&client, &rustlang_req).await?;
    println!("{:?}", response);
    Ok(())
}
