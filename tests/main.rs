#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::{core::k256::ecdsa::SigningKey, prelude::*, types::U256, utils::Ganache};
use futures::future::Future;
use linkme::{distributed_slice, DistributedSlice};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, sync::Arc, time::Duration};

mod harness;
use crate::harness::types::{*, Client, Action};

mod tests;
use crate::tests::prestate::*;

pub type RunnerType = HookRunner<DevRpcHooks<Client>>;

#[tokio::main]
async fn main() -> Result<()> {
    let node = Ganache::new().spawn();
    let provider = Provider::<Http>::try_from(node.endpoint())?.interval(Duration::from_millis(1));
    let accts: Vec<LocalWallet> = node.keys()[..5].iter().map(|x| x.clone().into()).collect();
    let client = Arc::new(DevRpcMiddleware::new(SignerMiddleware::new(
        provider,
        accts[0].clone(),
    )));

    let hooks = DevRpcHooks::new(client.clone());
    let runner = HookRunner::new(hooks);
    let state = DevRpcInitState { client: client, accts: accts };
    runner.start::<DevRpcInitState, DevRpcInitState>(state).await?;
    Ok(())
}
