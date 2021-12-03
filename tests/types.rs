use anyhow::Result;
use ethers::{core::k256::ecdsa::SigningKey, prelude::*};
use futures::future::Future;
use linkme::distributed_slice;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub type Client = DevRpcMiddleware<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;
pub type AsyncResult = std::pin::Pin<Box<dyn Future<Output = Result<()>>>>;
pub type Action<T> = fn(T) -> AsyncResult;

pub struct Test<T> {
    pub name: &'static str,
    pub run: Action<T>,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
    pub simple: SimpleState<Client>,
}

#[distributed_slice]
pub static TESTS: [Test<Context>] = [..];

abigen!(
    SimpleState,
    r#"[
        function state() external view returns (uint256)
        function move(uint256) external view returns (uint256)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);
