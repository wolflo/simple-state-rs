use anyhow::Result;
use futures::future::Future;
use once_cell::sync::Lazy;
use ethers::{prelude::*, core::k256::ecdsa::SigningKey};
use std::sync::Arc;
use linkme::distributed_slice;

pub type Client = DevRpcMiddleware<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;
pub type AsyncResult = std::pin::Pin<Box<dyn Future<Output=Result<()>>>>;
pub type Test<T> = fn(T) -> AsyncResult;

#[derive(Debug, Clone)]
pub struct Context {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
    pub simple: SimpleState<Client>,
}

#[distributed_slice]
pub static TESTS: [Test<Context, >] = [..];

abigen!(
    SimpleState,
    r#"[
        function state() external view returns (uint256)
        function move(uint256) external view returns (uint256)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);
