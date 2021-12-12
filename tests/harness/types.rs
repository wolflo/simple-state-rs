#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::{core::k256::ecdsa::SigningKey, prelude::*, types::U256, utils::Ganache};
use futures::future::Future;
use linkme::{distributed_slice, DistributedSlice};
use std::{convert::TryFrom, sync::Arc, time::Duration};

use crate::RunnerType;

pub type Client = DevRpcMiddleware<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;
pub type AsyncResult<R> = std::pin::Pin<Box<dyn Future<Output = R> + Send>>;
pub type Action<T> = fn(T) -> AsyncResult<Result<()>>;
pub type StateMove<S, R> = fn(S, R) -> AsyncResult<Result<()>>;

#[distributed_slice]
pub static STATES_FROM_INIT_STATE: [StateMove<DevRpcInitState, RunnerType>] = [..];

#[distributed_slice]
pub static TESTS_ON_INIT_STATE: [Test<DevRpcInitState>] = [..];

// Defines the state passed into each test
#[async_trait]
pub trait State: Clone + Send + Sync {
    type Base: State;
    async fn new(base: Self::Base) -> Result<Self>;
}
pub struct Test<S> {
    pub name: &'static str,
    pub run: Action<S>,
}
pub trait TestSet {
    type State: State;
    type Runner: Runner;
    fn tests(&self) -> &'static [Test<Self::State>] {
        &[]
    }
    fn children(&self) -> &'static [StateMove<Self::State, Self::Runner>] {
        &[]
    }
}

pub async fn dispatch<B, S, R>(base_state: B, runner: R) -> Result<()>
where
    B: Send + Sync,
    S: 'static + State<Base = B> + TestSet<State = S, Runner = R>,
    R: Runner,
{
    runner.start::<B, S>(base_state).await
}

#[async_trait]
pub trait Runner: Clone {
    async fn start<'s, B, S>(&'s self, base_state: B) -> Result<()>
    where
        B: Send + Sync,
        S: 'static + State<Base = B> + TestSet<State = S, Runner = Self>;
}

#[async_trait]
pub trait Hooks: Clone + Send + Sync {
    async fn before(&mut self) -> Result<()> {
        Ok(())
    }
    async fn before_each(&mut self) -> Result<()> {
        Ok(())
    }
    async fn after_each(&mut self) -> Result<()> {
        Ok(())
    }
    async fn after(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HookRunner<H: Hooks> {
    pub hooks: H,
}
#[async_trait]
impl<H: 'static + Hooks> Runner for HookRunner<H> {
    async fn start<'s, B, S>(&'s self, base_state: B) -> Result<()>
    where
        B: Send + Sync,
        S: 'static + State<Base = B> + TestSet<State = S, Runner = Self>,
    {
        let mut hooks = self.hooks.clone();
        hooks.before().await?;
        let state = S::new(base_state).await?;
        let tests = state.tests();
        let children = state.children();
        self.clone().run_children(state.clone(), children).await?;
        self.run_tests(state, tests).await?;
        hooks.after().await?;
        Ok(())
    }
}
impl<H: Hooks> HookRunner<H> {
    pub fn new(hooks: H) -> Self {
        Self { hooks }
    }
    async fn run_children<S: Clone + Send + Sync>(
        self,
        state: S,
        children: &[StateMove<S, Self>],
    ) -> Result<()> {
        println!("running {} children.", children.len());
        for dispatch in children {
            let mut hooks = self.hooks.clone();
            hooks.before_each().await?;
            dispatch(state.clone(), self.clone()).await?;
            hooks.after_each().await?;
        }
        Ok(())
    }
    async fn run_child<S: Send + Sync, N>(&self, state: S, child: &StateMove<S, N>) -> Result<()> {
        Ok(())
    }
    async fn run_tests<S: Clone + Send + Sync>(&self, state: S, tests: &[Test<S>]) -> Result<()> {
        for t in tests {
            println!("{}", t.name);
            let mut hooks = self.hooks.clone();
            hooks.before_each().await?;
            (t.run)(state.clone()).await?;
            hooks.after_each().await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DevRpcHooks<M: DevMiddleware + Clone> {
    snap_id: U256,
    client: Arc<M>,
}
impl<M: DevMiddleware + Clone> DevRpcHooks<M> {
    pub fn new(client: Arc<M>) -> Self {
        Self {
            snap_id: U256::from(0),
            client,
        }
    }
}
#[async_trait]
impl<M: 'static + DevMiddleware + Clone> Hooks for DevRpcHooks<M> {
    async fn before_each(&mut self) -> Result<()> {
        self.snap_id = self.client.snapshot().await?;
        Ok(())
    }
    async fn after_each(&mut self) -> Result<()> {
        self.client.reset(self.snap_id).await?;
        self.snap_id = self.client.snapshot().await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DevRpcInitState {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
}
#[async_trait]
impl State for DevRpcInitState {
    type Base = DevRpcInitState;
    async fn new(base: Self::Base) -> Result<Self> {
        Ok(base.clone())
    }
}
impl TestSet for DevRpcInitState {
    type State = DevRpcInitState;
    type Runner = RunnerType;
    fn tests(&self) -> &'static [Test<Self::State>] {
        &TESTS_ON_INIT_STATE
    }
    fn children(&self) -> &'static [StateMove<Self::State, Self::Runner>] {
        &STATES_FROM_INIT_STATE
    }
}
