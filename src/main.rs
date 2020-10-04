#[warn(clippy::all)]
mod query_client;
mod query_structs;

use graphql_client::QueryBody;
use query_client::QueryClient;
use query_structs::repoview::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rustlang_req = repoview_request("rust-lang", "rust", 50, None);
    let client = QueryClient::new()?; //expect("Client failed. TLS?");
    let response = client
        .request::<QueryBody<repo_view::Variables>, RepoView>(&rustlang_req)
        .await?;

    if let Some(data) = response.data {
        println!("{:#?}", data);
    }

    if let Some(errors) = response.errors {
        println!("{:#?}", errors);
    }

    Ok(())
}
