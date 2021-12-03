use anyhow::{anyhow, Result};
use ethers::utils::{Ganache, GanacheInstance};
use futures::{self, FutureExt};
use std::panic::AssertUnwindSafe;

mod tests;
mod types;
use tests::setup;
use types::{Test, TESTS_BASE, TESTS_CTX1, Context1};

#[tokio::main]
async fn main() -> Result<()> {
    let mut failures = 0;
    let mut successes = 0;
    let node: GanacheInstance = Ganache::new().spawn();
    let ctx = setup(&node, 3).await?;

    let (s, f) = run(ctx.clone(), &TESTS_BASE).await?;
    successes += s;
    failures += f;

    let ctx1: Context1 = ctx.into();
    let (s, f) = run(ctx1, &TESTS_CTX1).await?;
    successes += s;
    failures += f;

    println!("{} succeeded. {} failed", successes, failures);
    anyhow::ensure!(failures == 0, "Test failure.");
    Ok(())
}

async fn run<T: Clone>(ctx: T, tests: &'static [Test<T>]) -> Result<(usize, usize)> {
    let mut failures = 0;
    let mut successes = 0;
    for t in tests {
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
    Ok((successes, failures))

}
