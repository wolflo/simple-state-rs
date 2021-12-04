use anyhow::{anyhow, Result};
use ethers::utils::{Ganache, GanacheInstance};
use futures::{self, FutureExt};
use std::panic::AssertUnwindSafe;
use once_cell::sync::Lazy;

mod scratch;
use scratch::*;
mod tests;
mod types;
// use tests::setup;
// use types::*;

#[tokio::main]
async fn main() -> Result<()> {
    for tset in TSETS {
        println!("\nSet");
        for t in tset.tests() {
            println!("{}", t.name);
        }
    }

    // let mut failures = 0;
    // let mut successes = 0;
    // let node: GanacheInstance = Ganache::new().spawn();
    // let ctx = setup(&node, 3).await?;

    // let (s, f) = run(&mut ctx.clone(), &TESTS_BASE).await?;
    // successes += s;
    // failures += f;

    // let ctx1: Context1 = ctx.into();
    // let (s, f) = run(&mut ctx1.clone(), &TESTS_CTX1).await?;
    // successes += s;
    // failures += f;

    // println!("{} succeeded. {} failed", successes, failures);
    // anyhow::ensure!(failures == 0, "Test failure.");
    // Ok(())

    // really we need a set of tests rather than a type for each "from" context
    // for each context, we need a set of tests + an iterator over sets of tests
    // that can be applied to context that can be generated from our context.
    // Want to create a list of lists of tests, where each set of tests acts on
    // a ctx that can be created from current ctx.
    // I think current problem is enum is being allocated but we need this to be dyn
    // for l in types::LISTS {
    //     for x in **l {
    //         println!("x: {}", x);
    //     }
    // }
    // static stat: DS<Test<dyn BuildFromContext<BaseContext>>> = DS(TESTS_CTX1) as DS<Test<dyn BuildFromContext<BaseContext>>>;
    // static stat: DS<Test<FromBaseContext>> = DS(TESTS_CTX1.into());
    // let foo = stat.0; // this overflows stack
    // for x in types::FROM_BASE {
    //     println!("Enter");
    //     for y in x.0.static_slice() {
    //         println!("FROM_BASE");
    //     }
    // }
    Ok(())
}

// async fn run<T: Context + Clone>(ctx: &mut T, tests: &'static [Test<T>]) -> Result<(usize, usize)> {
//     let mut failures = 0;
//     let mut successes = 0;
//     for t in tests {
//         let res = AssertUnwindSafe((t.run)(ctx.clone()))
//             .catch_unwind()
//             .await
//             .unwrap_or(Err(anyhow!("Test Panic.")));
//         match res {
//             Ok(_) => {
//                 successes += 1;
//                 println!("test passed: {}", t.name);
//             }
//             Err(e) => {
//                 failures += 1;
//                 println!("\ntest failed: {} \n\tError: {:?}\n", t.name, e);
//             }
//         }
//         ctx.reset().await?;
//     }
//     Ok((successes, failures))
// }
