use anyhow::Result;
use ethers::{prelude::*, utils::GanacheInstance};
use linkme::distributed_slice;
use std::{convert::TryFrom, fs, path::Path, sync::Arc, time::Duration};

use crate::types::*;

const BUILD_DIR: &'static str = "out";

const _: () = {
    #[distributed_slice(TESTS_CTX1)]
    static __: Test<Context1> = Test {
        name: "test_wannacry",
        run: |x| Box::pin(test_wannacry(x)),
    };
};
const _: () = {
    #[distributed_slice(TESTS_BASE)]
    static __: Test<BaseContext> = Test {
        name: "test_step_from0",
        run: |x| Box::pin(test_step_from0(x)),
    };
};
const _: () = {
    #[distributed_slice(TESTS_BASE)]
    static __: Test<BaseContext> = Test {
        name: "test_setup",
        run: |x| Box::pin(test_setup(x)),
    };
};

pub async fn test_setup(ctx: BaseContext) -> Result<()> {
    assert_eq!(ctx.state.state().call().await?, 0.into());
    Ok(())
}
pub async fn test_step_from0(ctx: BaseContext) -> Result<()> {
    ctx.state.step(1.into()).send().await?;
    assert_eq!(ctx.state.state().call().await?, 1.into());
    Ok(())
}
pub async fn test_wannacry(ctx: Context1) -> Result<()> {
    ctx.state.step(1.into()).send().await?;
    ctx.state.step(3.into()).send().await?;
    ctx.state.step(5.into()).send().await?;
    ctx.state.wannacry(ctx.null.address()).send().await?;
    ctx.state.step(6.into()).send().await?;
    assert_eq!(ctx.state.state().call().await?, 6.into());
    Ok(())
}

pub async fn setup(node: &GanacheInstance, n_accts: usize) -> Result<BaseContext> {
    let provider = Provider::<Http>::try_from(node.endpoint())?.interval(Duration::from_millis(1));
    let accts: Vec<LocalWallet> = node.keys()[..n_accts]
        .iter()
        .map(|x| x.clone().into())
        .collect();
    let client = Arc::new(DevRpcMiddleware::new(SignerMiddleware::new(
        provider,
        accts[0].clone(),
    )));
    let factory = make_factory("SimpleState", &client)?;
    let deployed = factory.deploy(())?.send().await?;
    let state = SimpleState::new(deployed.address(), client.clone());
    let snap_id = client.snapshot().await?;
    Ok(BaseContext {
        snap_id,
        client,
        accts,
        state,
    })
}
#[derive(serde::Deserialize)]
struct FoundryOutput {
    abi: ethers::abi::Abi,
    bin: ethers::types::Bytes,
}
pub fn make_factory<M: Middleware>(name: &str, client: &Arc<M>) -> Result<ContractFactory<M>> {
    let name = String::from(name);
    let build_dir = Path::new(BUILD_DIR);

    let json = fs::read_to_string(
        &build_dir
            .join(name.clone() + ".sol")
            .join(name.clone() + ".json"),
    )?;
    let contract: FoundryOutput = serde_json::from_str(&json)?;

    Ok(ContractFactory::new(
        contract.abi,
        contract.bin,
        client.clone(),
    ))
}
