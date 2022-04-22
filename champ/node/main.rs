use anyhow::Result;
use tokio::runtime;

fn main() -> Result<()> {
    let rt = runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    rt.block_on(async { champ_node::run().await })
}
