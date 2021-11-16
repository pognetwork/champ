use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    champ_node::run().await
}
