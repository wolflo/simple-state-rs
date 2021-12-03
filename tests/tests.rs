use anyhow::Result;
use std::{convert::TryFrom, fs, path::Path, sync::Arc, time::Duration};
use ethers::{prelude::*, utils::GanacheInstance};
use linkme::distributed_slice;

use crate::types::*;

const BUILD_DIR: &'static str = "out";

pub async fn test_setup(ctx: Context) -> Result<()> {
    println!("running test_setup");
    Ok(())
}

const _: () = {
    #[distributed_slice(TESTS)]
    static __: Test<Context> = |x| Box::pin(test_setup(x));
};

const _: () = {
    #[distributed_slice(TESTS)]
    static __: Test<Context> = |x| Box::pin(test_move_from0(x));
};

pub async fn test_move_from0(ctx: Context) -> Result<()> {
    println!("running test_move_from0");
    Ok(())
}

pub async fn setup(node: &GanacheInstance, n_accts: usize) -> Result<Context> {
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
    let deployed = factory
        .deploy(())?
        .send()
        .await?;
    let simple = SimpleState::new(deployed.address(), client.clone());
    Ok(Context {
        client,
        accts,
        simple,
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

    let json = fs::read_to_string(&build_dir.join(name.clone() + ".sol").join(name.clone() + ".json"))?;
    let contract: FoundryOutput = serde_json::from_str(&json)?;

    Ok(ContractFactory::new(contract.abi, contract.bin, client.clone()))
}
