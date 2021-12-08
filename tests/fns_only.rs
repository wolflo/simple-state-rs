#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{convert::TryFrom, time::Duration, sync::Arc};
use anyhow::{anyhow, Result};
use futures::future::Future;
use async_trait::async_trait;
use linkme::{distributed_slice, DistributedSlice};
use ethers::{
    core::k256::ecdsa::SigningKey,
    types::U256,
    utils::Ganache,
    prelude::*,
};

pub async fn gmain() -> Result<()> {
    dispatch_old::<NullState, NullState, NullBlock>(&NullState).await.unwrap();

    // let hooks = DecRpcHooks::new();
    // let runner = HookRunner::new(&hooks);
    // dispatch::<NullState, NullState, >(&NullState).await.unwrap();

    // let runner = HookRunner::new(&DefaultHooks);

    Ok(())
    // run_state(NullS, &STATES_FROM_NULL_STATE, &FNS_ON_NULL_STATE);
    // dispatch::<S1>
}
#[async_trait]
pub trait Hooks: Send + Sync {
    async fn before_each(&mut self) -> Result<()> { Ok(()) }
    async fn after_each(&mut self) -> Result<()> { Ok(()) }
    async fn after(&mut self) -> Result<()> { Ok(()) }
}
#[async_trait]
pub trait Runner {
    async fn run<'s, S: Sync>(&mut self, state: &'s S, tests: &[Test<&'s S>], children: &[Action<&'s S>]) -> Result<()>;
}
struct DevRpcHooks<M: Middleware> {
    snap_id: U256,
    client: DevRpcMiddleware<M>,
}
impl<M: Middleware> DevRpcHooks<M> {
    fn new(client: DevRpcMiddleware<M>) -> Self { Self { snap_id: U256::from(0), client }}
}
#[async_trait]
impl<M: 'static + Middleware> Hooks for DevRpcHooks<M> {
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
struct HookRunner<'h, H: Hooks> {
    hooks: &'h mut H,
}
impl<'h, H: Hooks> HookRunner<'h, H> {
    pub fn new(hooks: &'h mut H) -> Self { Self { hooks } }
    async fn run_tests<'s, S: Sync>(&mut self, state: &'s S, tests: &[Test<&'s S>]) -> Result<()> {
        for t in tests {
            println!("{}", t.name);
            self.hooks.before_each().await?;
            (t.run)(state).await?;
            self.hooks.after_each().await?;
        }
        Ok(())
    }
    async fn run_children<'s, S: Sync>(&mut self, state: &'s S, children: &[Action<&'s S>]) -> Result<()> {
        for runner in children {
            self.hooks.before_each().await?;
            runner(state).await?;
            self.hooks.after_each().await?;
        }
        Ok(())
    }
}
#[async_trait]
impl<H: Hooks> Runner for HookRunner<'_, H> {
    async fn run<'s, S: Sync>(&mut self, state: &'s S, tests: &[Test<&'s S>], children: &[Action<&'s S>]) -> Result<()> {
        self.run_children(state, children).await?;
        self.run_tests(state, tests).await?;
        self.hooks.after().await
    }
}

pub async fn dispatch<R, P, S>(runner: &mut R, prev_state: &P) -> Result<()>
where
    R: Runner,
    S: State<Prev=P> + TestSet<State=S>,
{
    let state = S::new(prev_state).await?;
    let tests = S::tests();
    let children = S::children();
    runner.run(&state, &tests, &children).await?;
    Ok(())
}
// runner is a struct that takes takes the tests as args
// a Runner needs to wrap a Hooks. they both take in state, tests, children as args
// pub async fn run<S: State, R>(state: S, runner: R, children: &[Action<S>], tests: &[Test<S>]) -> Result<()> { }

pub type Client = DevRpcMiddleware<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;
pub type AsyncResult = std::pin::Pin<Box<dyn Future<Output = Result<()>> + Send>>;
pub type Action<T> = fn(T) -> AsyncResult;
#[derive(Debug, Clone)]
pub struct NullState;
#[async_trait]
impl State for NullState {
    type Prev = NullState;
    async fn new(_prev: &Self::Prev) -> Result<Self> { Ok(NullState) }
}
#[derive(Debug)]
pub struct NullBlock { pub state: NullState, }
#[async_trait]
impl Block for NullBlock {
    type State = NullState;
    fn new(state: Self::State) -> Self { Self { state } }
    fn state(&self) -> &Self::State { &self.state }
    fn tests(&self) -> &'static [Test<&Self::State>] { &NULL_STATE_TESTS }
    fn children(&self) -> &'static [Action<&Self::State>] { &STATES_FROM_NULL_STATE }
}
impl HooksOld for NullBlock {}
impl RunnerOld for NullBlock {}
#[distributed_slice]
pub static STATES_FROM_NULL_STATE: [Action<&'static NullState>] = [..];
#[distributed_slice]
pub static NULL_STATE_TESTS: [Test<&'static NullState>] = [..];
pub struct Test<S> {
    pub name: &'static str,
    pub run: Action<S>,
}
pub async fn dispatch_old<P, S, R>(prev_state: &P) -> Result<()>
where
    S: State<Prev=P>,
    R: RunnerOld<State=S> + Sync
{
    let state: S = S::new(prev_state).await.unwrap();
    let runner: R = R::new(state);
    runner.run().await
}

// --- User defined
#[derive(Debug, Clone)]
pub struct BaseState {
    pub snap_id: Option<U256>,
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
}
#[async_trait]
impl State for BaseState {
    type Prev = NullState;
    async fn new(prev: &Self::Prev) -> Result<Self> {
        println!("Building BaseState");
        let node = Ganache::new().spawn();
        let provider = Provider::<Http>::try_from(node.endpoint())?.interval(Duration::from_millis(1));
        let accts: Vec<LocalWallet> = node.keys()[..5]
            .iter()
            .map(|x| x.clone().into())
            .collect();
        let client = Arc::new(DevRpcMiddleware::new(SignerMiddleware::new(
            provider,
            accts[0].clone(),
        )));
        let snap_id = None;
        Ok(Self {
            snap_id,
            client,
            accts,
        })
    }
}

// --- Macro defined
#[distributed_slice]
pub static BASE_STATE_TESTS: [Test<&'static BaseState>] = [..];
#[distributed_slice]
pub static STATES_FROM_BASE_STATE: [Action<&'static BaseState>] = [..];
#[distributed_slice(STATES_FROM_NULL_STATE)]
static __SN1: Action<&NullState> = |x| Box::pin(dispatch_old::<NullState, BaseState, BaseBlock>(&x));
#[derive(Debug)]
pub struct BaseBlock { pub state: BaseState, }
#[async_trait]
impl Block for BaseBlock {
    type State = BaseState;
    fn new(state: Self::State) -> Self { Self { state } }
    fn state(&self) -> &Self::State { &self.state }
    fn tests(&self) -> &'static [Test<&Self::State>] { &BASE_STATE_TESTS }
    fn children(&self) -> &'static [Action<&Self::State>] { &STATES_FROM_BASE_STATE }
}
impl HooksOld for BaseBlock {}
impl RunnerOld for BaseBlock {}



#[async_trait]
pub trait State: Clone + Send + Sync {
    type Prev: State;
    async fn new(prev: &Self::Prev) -> Result<Self>;
}
pub trait Block {
    type State: State;
    fn new(state: Self::State) -> Self;
    fn state(&self) -> &Self::State;
    fn tests(&self) -> &[Test<&Self::State>] { &[] }
    fn children(&self) -> &[Action<&Self::State>] { &[] }
}
#[async_trait]
pub trait HooksOld: Block {
    async fn before_each(&self) -> Result<()> { Ok(()) }
    async fn after_each(&self) -> Result<()> { Ok(()) }
    async fn after(&self) -> Result<()> { Ok(()) }
}
#[async_trait]
pub trait RunnerOld: HooksOld {
    async fn run_tests(&self) -> Result<()> {
        for t in self.tests() {
            println!("{}", t.name);
            self.before_each().await?;
            (t.run)(self.state()).await?;
            self.after_each().await?;
        }
        Ok(())
    }
    async fn run_children(&self) -> Result<()> {
        for runner in self.children() {
            self.before_each().await?;
            runner(self.state()).await?;
            self.after_each().await?;
        }
        Ok(())
    }
    async fn run(&self) -> Result<()> {
        self.run_children().await?;
        self.run_tests().await?;
        self.after().await
    }
}

pub trait TestSet {
    type State: State;
    // fn tests() -> &'a [Test<&'a Self::State>] { &[] }
    // fn children() -> &'a [Action<&'a Self::State>] { &[] }
    // fn tests() -> &'a [Test<&'s Self::State>] { &[] }
    fn tests<'a>() -> &'a [Test<&'a Self::State>] { &[] }
    fn children<'a>() -> &'a [Action<&'a Self::State>] { &[] }
}
// impl TestSet for BaseState {
//     type State = BaseState;
//     fn tests() -> &'static [Test<&'static Self::State>] { &BASE_STATE_TESTS }
//     fn children() -> &'static [Action<&'static Self::State>] { &STATES_FROM_BASE_STATE }
// }

// pub trait StateFrom<T> { fn new(src: T) -> Self; }
// impl<T> StateFrom<T> for T { fn new(src: T) -> Self { src }}

// #[derive(Debug, Clone)]
// pub struct NullS;
// impl DevClient for NullS {}
// #[distributed_slice]
// pub static FNS_ON_NULL_STATE: [fn(NullS)] = [..];
// #[distributed_slice]
// pub static STATES_FROM_NULL_STATE: [fn(NullS)] = [..];

// pub trait DevClient {
//     // client() needs to be impl by users so we can provide default
//     // impls for snap and reset?
//     // fn client(&self) -> DevRpcMiddleware;
//     fn snap(&self) -> u64 { 17 }
//     fn reset(&self, id: u64) {}
// }

// #[derive(Debug, Clone)]
// pub struct S1;
// impl StateFrom<NullS> for S1 { fn new(s: NullS) -> Self { S1 } }

// // macro generated:
// impl DevClient for S1 {}
// #[distributed_slice]
// pub static FNS_ON_S1: [fn(S1)] = [..];
// #[distributed_slice]
// pub static STATES_FROM_S1: [fn(S1)] = [..];
// #[distributed_slice(STATES_FROM_NULL_STATE)]
// static _S01: fn(NullS) = run_s1;
// fn run_s1(b: NullS) { run_state(b, &STATES_FROM_S1, &FNS_ON_S1) }

// #[derive(Debug, Clone)]
// pub struct S2;
// impl StateFrom<S1> for S2 { fn new(s: S1) -> Self { S2 } }

// impl DevClient for S2 {}
// #[distributed_slice]
// pub static FNS_ON_S2: [fn(S2)] = [..];
// #[distributed_slice]
// pub static STATES_FROM_S2: [fn(S2)] = [..];
// #[distributed_slice(STATES_FROM_S1)]
// static _S11: fn(S1) = run_s2;
// fn run_s2(b: S1) { run_state(b, &STATES_FROM_S2, &FNS_ON_S2) }

// #[distributed_slice(FNS_ON_S1)]
// static _T11: fn(S1) = act_on_s1a;
// pub fn act_on_s1a(_: S1) { println!("act_on_s1a"); }
// #[distributed_slice(FNS_ON_S1)]
// static _T12: fn(S1) = act_on_s1b;
// pub fn act_on_s1b(_: S1) { println!("act_on_s1b"); }
// #[distributed_slice(FNS_ON_S2)]
// static _T21: fn(S2) = act_on_s2a;
// pub fn act_on_s2a(_: S2) { println!("act_on_s2a"); }
// #[distributed_slice(FNS_ON_S2)]
// static _T22: fn(S2) = act_on_s2b;
// pub fn act_on_s2b(_: S2) { println!("act_on_s2b"); }

// fn run_state<B, S>(base: B, sub_states: &[fn(S)], tests: &[fn(S)])
// where
//     S: StateFrom<B> + DevClient + Clone
// {
//     let state: S = S::new(base);
//     let mut snap_id = state.snap();
//     for runner in sub_states {
//         runner(state.clone());
//         state.reset(snap_id);
//         snap_id = state.snap();
//     }
//     for t in tests {
//         t(state.clone());
//         state.reset(snap_id);
//         snap_id = state.snap();
//     }
// }
// would need to take the array of child states and
// fn run_<S: From<B>, B>(base: B) {
//     let state: S = S::from(base);
//     for run_sub_state in STATES_FROM_NULL {
//         run_sub_state(state.clone())
//         state.reset();
//     }
//     for f in FNS_ON_NULL_STATE
// }
// fn run_null(base: NullS) {
//     let state: NullS = NullS::state_from(base); // must store snap_id
//     let mut snap_id = state.snap();
//     for run_sub_state in STATES_FROM_NULL_STATE {
//         run_sub_state(state.clone());
//         state.reset(snap_id);
//         snap_id = state.snap();
//     }
//     for f in FNS_ON_NULL_STATE {
//         f(state.clone());
//         state.reset(snap_id);
//         snap_id = state.snap();
//     }
// }
// fn run_s1(base: NullS) {
//     let state: S1 = S1::state_from(base); // must store snap_id
//     let mut snap_id = state.snap();
//     for run_sub_state in STATES_FROM_S1 {
//         run_sub_state(state.clone());
//         state.reset(snap_id);
//         snap_id = state.snap();
//     }
//     for f in FNS_ON_S1 {
//         f(state.clone());
//         state.reset(snap_id);
//         snap_id = state.snap();
//     }
// }

