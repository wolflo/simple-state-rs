pub use crate::harness::types::{dispatch, Client, State, StateMove, Test, TestSet};
pub use crate::RunnerType;
pub use anyhow::{anyhow, Result};
pub use async_trait::async_trait;
pub use linkme::{distributed_slice, DistributedSlice};
pub use ethtest::ethtest;

pub use ethers::{prelude::LocalWallet, providers::Middleware, signers::Signer};
use ethers::{
    prelude::{abigen, ContractFactory},
    types::Bytes,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::Arc};

pub const BUILD_DIR: &'static str = env!("SOLC_BUILD_DIR");

abigen!(
    SimpleState,
    "./build/SimpleState.abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub fn make_factory<M: Middleware>(name: &str, client: &Arc<M>) -> Result<ContractFactory<M>> {
    let build_dir = Path::new(BUILD_DIR);
    let abi_raw = fs::read_to_string(&build_dir.join(name.to_owned() + ".abi.json"))?;
    let bin_raw = fs::read_to_string(&build_dir.join(name.to_owned() + ".bin.json"))?;
    let abi = serde_json::from_str(&abi_raw)?;
    let bytecode = hex::decode(&bin_raw)?.into();
    let factory = ContractFactory::new(abi, bytecode, client.clone());
    Ok(factory)
}
