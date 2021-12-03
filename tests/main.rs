use anyhow::Result;
use ethers::{
    utils::{Ganache, GanacheInstance},
};

mod types;
mod tests;
use types::TESTS;
use tests::setup;

#[tokio::main]
async fn main() -> Result<()> {
    let node: GanacheInstance = Ganache::new().spawn();
    let ctx = setup(&node, 3).await?;
    for t in TESTS {
        t(ctx.clone()).await?;
    }
    Ok(())
}
