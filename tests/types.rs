use anyhow::Result;
use ethers::{core::k256::ecdsa::SigningKey, prelude::*};
use futures::future::Future;
use futures_executor::block_on;
use linkme::distributed_slice;
use once_cell::sync::Lazy;
use std::sync::Arc;

use crate::tests::make_factory;

pub type Client = DevRpcMiddleware<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;
pub type AsyncResult = std::pin::Pin<Box<dyn Future<Output = Result<()>>>>;
pub type Action<T> = fn(T) -> AsyncResult;

pub struct Test<T> {
    pub name: &'static str,
    pub run: Action<T>,
}

#[distributed_slice]
pub static TESTS_BASE: [Test<BaseContext>] = [..];
#[distributed_slice]
pub static TESTS_CTX1: [Test<Context1>] = [..];

#[derive(Debug, Clone)]
pub struct BaseContext {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
    pub state: SimpleState<Client>,
}

#[derive(Debug, Clone)]
pub struct Context1 {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
    pub state: SimpleState<Client>,
    pub null: NullContract<Client>,
}

impl From<BaseContext> for Context1 {
    fn from(ctx: BaseContext) -> Self {
        let factory = make_factory("NullContract", &ctx.client).unwrap();
        let deployed = block_on(factory.deploy(()).unwrap().send()).unwrap();
        let null = NullContract::new(deployed.address(), ctx.client.clone());
        Context1 {
            client: ctx.client,
            accts: ctx.accts,
            state: ctx.state,
            null: null
        }
    }
}


abigen!(
    SimpleState,
    r#"[
        function state() external view returns (uint256)
        function step(uint256) external view returns (uint256)
        function wannacry(address) external
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);
abigen!(
    NullContract,
    r#"[]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);
