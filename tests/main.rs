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

// Can we use a generic fn (i.e. a function that allows us to hide the type signature from
// rust) to construct the type we want (rather than to construct a value of the type we pass)?

pub type Client = DevRpcMiddleware<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;
pub type AsyncResult<R> = std::pin::Pin<Box<dyn Future<Output = R> + Send>>;
pub type Action<T> = fn(T) -> AsyncResult<Result<()>>;
pub type StateMove<S, R> = fn(S, R) -> AsyncResult<Result<()>>;
type RunnerType = HookRunner<DevRpcHooks<Client>>;

fn main() {}

pub async fn gmain() -> Result<()> {
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

#[derive(Debug, Clone)]
pub struct DevRpcInitState {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
}
#[async_trait]
impl State for DevRpcInitState {
    type Prev = DevRpcInitState;
    async fn new(prev: Self::Prev) -> Result<Self> {
        Ok(prev.clone())
    }
}
pub struct Test<S> {
    pub name: &'static str,
    pub run: Action<S>,
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
#[async_trait]
pub trait Runner: Clone {
    async fn start<'s, P, S>(&'s self, prev_state: P) -> Result<()>
    where
        P: Send + Sync,
        S: 'static + State<Prev = P> + TestSet<State = S, Runner = Self>;
}
#[derive(Debug, Clone)]
pub struct DevRpcHooks<M: DevMiddleware + Clone> {
    snap_id: U256,
    client: Arc<M>,
}
impl<M: DevMiddleware + Clone> DevRpcHooks<M> {
    fn new(client: Arc<M>) -> Self {
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
pub struct HookRunner<H: Hooks> {
    pub hooks: H,
}
#[async_trait]
impl<H: 'static + Hooks> Runner for HookRunner<H> {
    async fn start<'s, P, S>(&'s self, prev_state: P) -> Result<()>
    where
        P: Send + Sync,
        S: 'static + State<Prev = P> + TestSet<State = S, Runner = Self>,
    {
        let mut hooks = self.hooks.clone();
        hooks.before().await?;
        let state = S::new(prev_state).await?;
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
            dispatch(state.clone(), self.clone());
            hooks.after_each().await?;
        }
        Ok(())
    }
    async fn run_child<S: Send + Sync, N>(
        &self,
        state: S,
        child: &StateMove<S, N>,
    ) -> Result<()> {
        Ok(())
    }
    async fn run_tests<S: Clone + Send + Sync>(
        &self,
        state: S,
        tests: &[Test<S>],
    ) -> Result<()> {
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

#[async_trait]
pub trait State: Clone + Send + Sync {
    type Prev: State;
    async fn new(prev: Self::Prev) -> Result<Self>;
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

// --- User defined
#[derive(Debug, Clone)]
pub struct BaseState {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
}
#[async_trait]
impl State for BaseState {
    type Prev = DevRpcInitState;
    async fn new(prev: Self::Prev) -> Result<Self> {
        println!("Building BaseState");
        Ok(Self { client: prev.client, accts: prev.accts })
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
impl TestSet for BaseState {
    type State = BaseState;
    type Runner = RunnerType;
}

pub async fn dispatch<P, S, R>(prev_state: P, runner: R) -> Result<()>
where
    P: Send + Sync,
    S: 'static + State<Prev = P> + TestSet<State = S, Runner = R>,
    R: Runner,
{
    runner.start::<P, S>(prev_state).await
}

#[distributed_slice]
pub static STATES_FROM_INIT_STATE: [StateMove<DevRpcInitState, RunnerType>] = [..];
#[distributed_slice(STATES_FROM_INIT_STATE)]
pub static __SN1: StateMove<DevRpcInitState, RunnerType> =
    |s, r| Box::pin(dispatch::<DevRpcInitState, BaseState, RunnerType>(s, r));
#[distributed_slice]
pub static TESTS_ON_INIT_STATE: [Test<DevRpcInitState>] = [..];
#[distributed_slice(TESTS_ON_INIT_STATE)]
pub static __ST1: Test<DevRpcInitState> = Test { name: "init state test 1", run: |s| Box::pin(test_init_state1(s)) };
async fn test_init_state1(state: DevRpcInitState) -> Result<()> {
    Ok(())
}

// pub static FOO: [StatesFromNull; 0] = [|s| B::new(s)];
// #[distributed_slice]
// pub static STATES_FROM_NULL_STATE: [StateMove<&'static NullState>] = [..];
// #[distributed_slice]
// pub static TESTS_ON_NULL_STATE: [Test<&'static NullState>] = [..];
// #[distributed_slice]
// pub static TESTS_ON_BASE_STATE: [Test<&'static BaseState>] = [..];
// #[distributed_slice]
// pub static STATES_FROM_BASE_STATE: [StateMove<&'static BaseState>] = [..];
// #[distributed_slice(STATES_FROM_NULL_STATE)]
// static __SN1: StateMove<&NullState> = |s| Box::pin(move_state::<NullState, BaseState>(&s));

// async fn run<'s, S: Send + Sync>(&'s mut self, state: &'s S, tests: &[Test<&'s S>], children: &[Action<&'s S>]) -> Result<()> {
//     self.run_children(state, children).await?;
//     self.run_tests(state, tests).await?;
//     self.hooks.after().await
// }
// pub async fn move_state<'a, P, S>(prev_state: &P) -> Result<S>
// where
//     S: 'a + State<Prev=P> + TestSet<State=S>,
// {
//     S::new(prev_state).await
// }
// pub async fn dispatch<'a, R, P, S>(runner: &mut R, prev_state: &P) -> Result<()>
// where
//     R: Runner,
//     S: 'a + State<Prev=P> + TestSet<State=S>,
// {
//     let state = S::new(prev_state).await?;
//     let tests = S::tests();
//     let children = S::children();
//     runner.run(&state, &tests, &children).await?;
//     Ok(())
// }
// pub async fn start_runner<'a, R, P, S>(runner: &mut R, prev_state: &P) -> Result<()>
// where
//     R: Runner,
//     S: 'a + State<Prev=P> + TestSet<State=S>,
// {
//     let state = S::new(prev_state).await?;
//     let tests = S::tests();
//     let children = S::children();
//     runner.run(&state, &tests, &children).await?;
//     // runner.start(&state, &tests, &children).await?;
//     Ok(())
// }
