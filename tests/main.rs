use anyhow::{anyhow, Result};
use ethers::utils::{Ganache, GanacheInstance};
use futures::{self, FutureExt};
use std::panic::AssertUnwindSafe;
use once_cell::sync::Lazy;

mod tests;
mod types;
use tests::setup;
use types::{Context, Context1, Test, TESTS_BASE, TESTS_CTX1};

#[tokio::main]
async fn main() -> Result<()> {
    let mut failures = 0;
    let mut successes = 0;
    let node: GanacheInstance = Ganache::new().spawn();
    let ctx = setup(&node, 3).await?;

    let (s, f) = run(&mut ctx.clone(), &TESTS_BASE).await?;
    successes += s;
    failures += f;

    let ctx1: Context1 = ctx.into();
    let (s, f) = run(&mut ctx1.clone(), &TESTS_CTX1).await?;
    successes += s;
    failures += f;

    println!("{} succeeded. {} failed", successes, failures);
    anyhow::ensure!(failures == 0, "Test failure.");
    // Ok(())

    // really we need a set of tests rather than a type for each "from" context
    // for each context, we need a set of tests + an iterator over sets of tests
    // that can be applied to context that can be generated from our context
    for l in types::LISTS {
        for x in **l {
            println!("x: {}", x);
        }
    }
    for x in types::FROM_BASE {
        // println!("FROM_BASE");
        for y in x.0 {
            println!("FROM_BASE");
        }
    }
    Ok(())
}

async fn run<T: Context + Clone>(ctx: &mut T, tests: &'static [Test<T>]) -> Result<(usize, usize)> {
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
        ctx.reset().await?;
    }
    Ok((successes, failures))
}
