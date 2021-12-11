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

pub async fn gmain() -> Result<()> {
    let node = Ganache::new().spawn();
    let provider = Provider::<Http>::try_from(node.endpoint())?.interval(Duration::from_millis(1));
    let accts: Vec<LocalWallet> = node.keys()[..5].iter().map(|x| x.clone().into()).collect();
    let client = Arc::new(DevRpcMiddleware::new(SignerMiddleware::new(
        provider,
        accts[0].clone(),
    )));

    let mut hooks = DevRpcHooks::new(&client);
    let runner = HookRunner::new(&mut hooks);
    let state = DevRpcInit { client: &client, accts: &accts };
    runner.start::<DevRpcInit, DevRpcInit>(&state).await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct DevRpcInit<'a> {
    pub client: &'a Arc<Client>,
    pub accts: &'a Vec<LocalWallet>,
}
#[async_trait]
impl<'a> State for DevRpcInit<'a> {
    type Prev = DevRpcInit<'a>;
    async fn new(prev: &Self::Prev) -> Result<Self> {
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
    async fn start<'s, P, S>(&'s self, prev_state: &'s P) -> Result<()>
    where
        P: Sync,
        S: State<Prev = P> + TestSet<State = S>;
}
#[derive(Debug, Clone)]
pub struct DevRpcHooks<'m, M: Middleware + Clone> {
    snap_id: U256,
    client: &'m DevRpcMiddleware<M>,
}
impl<'m, M: Middleware + Clone> DevRpcHooks<'m, M> {
    fn new(client: &'m DevRpcMiddleware<M>) -> Self {
        Self {
            snap_id: U256::from(0),
            client,
        }
    }
}
#[async_trait]
impl<M: 'static + Middleware + Clone> Hooks for DevRpcHooks<'_, M> {
    async fn before_each(&mut self) -> Result<()> {
        self.snap_id = self.client.snapshot().await?;
        Ok(())
    }
    async fn after_each(&mut self) -> Result<()> {
        self.client.revert_to_snapshot(self.snap_id).await?;
        self.snap_id = self.client.snapshot().await?;
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct HookRunner<'h, H: Hooks> {
    hooks: &'h H,
}
#[async_trait]
impl<H: Hooks> Runner for HookRunner<'_, H> {
    async fn start<'s, P, S>(&'s self, prev_state: &'s P) -> Result<()>
    where
        P: Sync,
        S: State<Prev = P> + TestSet<State = S>,
    {
        let mut hooks = self.hooks.clone();
        hooks.before().await?;
        let state = S::new(prev_state).await?;
        let tests = S::tests();
        let children = S::children();
        self.clone().run_children(&state, children).await?;
        self.run_tests(&state, tests).await?;
        hooks.after().await?;
        Ok(())
    }
}
impl<'h, H: Hooks> HookRunner<'h, H> {
    pub fn new(hooks: &'h H) -> Self {
        Self { hooks }
    }
    async fn run_children<'s, S: Send + Sync>(
        self,
        state: &'s S,
        children: &[StateMove<&'s S, Self>],
    ) -> Result<()> {
        println!("running {} children.", children.len());
        for dispatch in children {
            let mut hooks = self.hooks.clone();
            hooks.before_each().await?;
            dispatch(state, self.clone());
            hooks.after_each().await?;
        }
        Ok(())
    }
    async fn run_child<'s, S: Send + Sync, N>(
        &self,
        state: &'s S,
        child: &StateMove<&'s S, N>,
    ) -> Result<()> {
        Ok(())
    }
    async fn run_tests<'s, S: Send + Sync>(
        &self,
        state: &'s S,
        tests: &[Test<&'s S>],
    ) -> Result<()> {
        for t in tests {
            println!("{}", t.name);
            let mut hooks = self.hooks.clone();
            hooks.before_each().await?;
            (t.run)(state).await?;
            hooks.after_each().await?;
        }
        Ok(())
    }
}

#[async_trait]
pub trait State: Clone + Send + Sync {
    type Prev: State;
    async fn new(prev: &Self::Prev) -> Result<Self>;
}

pub trait TestSet {
    type State: State;
    fn tests<'a>() -> &'a [Test<&'a Self::State>] {
        &[]
    }
    fn children<'a, R: Runner>() -> &'a [StateMove<&'a Self::State, R>] {
        &[]
    }
    fn tests_<'a>(&'a self) -> &'a [Test<&'a Self::State>] {
        &[]
    }
    // fn children_(&self) -> &'static [StateMove<&'static Self::State, RunnerType>] {
    //     &[]
    // }
}

// --- User defined
#[derive(Debug, Clone)]
pub struct BaseState<'a> {
    pub client: &'a Arc<Client>,
    pub accts: &'a Vec<LocalWallet>,
}
#[async_trait]
impl<'a> State for BaseState<'a> {
    type Prev = DevRpcInit<'a>;
    async fn new(prev: &Self::Prev) -> Result<Self> {
        println!("Building BaseState");
        Ok(Self { client: prev.client, accts: prev.accts })
    }
}

impl<'a> TestSet for DevRpcInit<'a> {
    type State = DevRpcInit<'a>;
    // fn children_(&self) -> &'static [StateMove<&'static Self::State, RunnerType>] {
    //     &STATES_FROM_INIT_STATE
    // }
}
impl<'a> TestSet for BaseState<'a> {
    type State = BaseState<'a>;
}

pub async fn dispatch<P, S, R>(prev_state: &P, runner: R) -> Result<()>
where
    P: Sync,
    S: State<Prev = P> + TestSet<State = S>,
    R: Runner,
{
    runner.start::<P, S>(prev_state).await
}

type RunnerType = HookRunner<'static, DevRpcHooks<'static, Provider<Http>>>;
#[distributed_slice]
pub static STATES_FROM_INIT_STATE: [StateMove<&'static DevRpcInit<'static>, RunnerType>] = [..];
#[distributed_slice(STATES_FROM_INIT_STATE)]
pub static __SN1: StateMove<&'static DevRpcInit<'static>, RunnerType> =
    |s, r| Box::pin(dispatch::<DevRpcInit<'static>, BaseState, RunnerType>(s, r));

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
