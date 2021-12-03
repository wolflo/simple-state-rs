use anyhow::{anyhow, Result};
use ethers::utils::{Ganache, GanacheInstance};
use futures::{self, FutureExt};
use std::panic::AssertUnwindSafe;

mod tests;
mod types;
use tests::setup;
use types::TESTS;

#[tokio::main]
async fn main() -> Result<()> {
    let node: GanacheInstance = Ganache::new().spawn();
    let ctx = setup(&node, 3).await?;
    let mut failures = 0;
    let mut successes = 0;
    for t in TESTS {
        let res = AssertUnwindSafe((t.run)(ctx.clone()))
            .catch_unwind()
            .await
            .unwrap_or(Err(anyhow!("Test Panic.")));
        match res {
            Ok(_) => {
                successes += 1;
                println!("test passed: {}", t.name);
            }
            Err(e) => {
                failures += 1;
                println!("\ntest failed: {} \n\tError: {:?}\n", t.name, e);
            }
        }
    }
    Ok(())
}
