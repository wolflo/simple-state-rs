use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::{core::k256::ecdsa::SigningKey, prelude::*};
use futures::future::Future;
use futures_executor::block_on;
use linkme::{distributed_slice, DistributedSlice};
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

#[distributed_slice]
pub static A: [usize] = [..];
#[distributed_slice(A)]
static a0: usize = 2;
#[distributed_slice(A)]
static a1: usize = 3;
#[distributed_slice]
pub static B: [usize] = [..];
#[distributed_slice(B)]
static b0: usize = 4;
#[distributed_slice(B)]
static b1: usize = 5;

#[distributed_slice]
pub static LISTS: [&'static DistributedSlice<[usize]>] = [..];
#[distributed_slice(LISTS)]
static l0: &'static DistributedSlice<[usize]> = &A;
#[distributed_slice(LISTS)]
static l1: &'static DistributedSlice<[usize]> = &B;

// struct X<Ctx> {
//     tests: [Tests<Ctx>],
//     next_tests: [ [Tests<Z>] ] where Z: From<Ctx>
// }

// #[distributed_slice]
// pub static TESTS_BASE: [Test<BaseContext>] = [..];
#[distributed_slice]
pub static TESTS_FROM_BASE: [Test<FromBaseContext>] = [..];
#[distributed_slice]
pub static FROM_BASE: [Lazy<DS<Test<&'static (dyn BuildFromContext<BaseContext>)>>>] = [..];
// pub static FROM_BASE: [Lazy<DST<FromBaseContext>>] = [..];
// pub static FROM_BASE: [DST<FromBaseContext>] = [..];
// pub static FROM_BASE: [&'static DistributedSlice<[Test<FromBaseContext>]>] = [..];
// pub static FROM_BASE: [[Tests<FromBaseContext>]];

// #[distributed_slice(LISTS)]
// static l1: &'static DistributedSlice<[usize]> = &B;
// #[distributed_slice(TESTS_FROM_BASE)]
// static _x0: &'static DistributedSlice<[usize]> = 


pub enum FromBaseContext {
    C1(Context1),
}
impl From<BaseContext> for FromBaseContext {
    fn from(ctx: BaseContext) -> Self {
        ctx.into()
    }
}
impl From<Context1> for FromBaseContext {
    fn from(ctx: Context1) -> Self {
        Self::C1(ctx)
    }
}
impl From<Test<Context1>> for Test<FromBaseContext> {
    fn from(t: Test<Context1>) -> Self {
        t.into()
    }
}
// pub struct DST<T>(pub DistributedSlice<[Test<T>]>);
// impl From<DistributedSlice<[Test<Context1>]>> for DST<FromBaseContext> {
//     fn from(d: DistributedSlice<[Test<Context1>]>) -> Self {
//         // d.map(|t| t.into())
//         d.into()
//     }
// }

#[async_trait]
pub trait Context {
    async fn reset(&mut self) -> Result<()>;
}
pub trait BuildFrom<T> {
    fn build_from(&mut self, t: T);
}
impl BuildFrom<BaseContext> for Context1 { fn build_from(&mut self, b: BaseContext) { } }
pub struct DS<T>(pub DistributedSlice<[T]>);
impl From<DistributedSlice<[Test<Context1>]>> for DS<Test<&'static (dyn BuildFromContext<BaseContext>)>> {
    fn from(s: DistributedSlice<[Test<Context1>]>) -> Self {
        s.into()
    }
}

#[derive(Debug, Clone)]
pub struct BaseContext {
    pub snap_id: ethers::types::U256,
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
    pub state: SimpleState<Client>,
}

#[async_trait]
impl Context for BaseContext {
    async fn reset(&mut self) -> Result<()> {
        self.client.revert_to_snapshot(self.snap_id).await
        .map_err(|_| anyhow!("Failed to reset snapshot."))?;
        self.snap_id = self.client.snapshot().await?;
        Ok(())
    }
}
#[async_trait]
impl Context for Context1 {
    async fn reset(&mut self) -> Result<()> {
        self.client.revert_to_snapshot(self.snap_id).await
        .map_err(|_| anyhow!("Failed to reset snapshot."))?;
        self.snap_id = self.client.snapshot().await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Context1 {
    pub snap_id: ethers::types::U256,
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
        let snap_id = block_on(ctx.client.snapshot()).unwrap();
        Context1 {
            snap_id: snap_id,
            client: ctx.client,
            accts: ctx.accts,
            state: ctx.state,
            null: null,
        }
    }
}
pub trait BuildFromContext<T>: BuildFrom<T> + Context + Send + Sync {}

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
