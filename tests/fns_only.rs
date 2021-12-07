#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::{anyhow, Result};
use futures::future::Future;
use async_trait::async_trait;
use linkme::{distributed_slice, DistributedSlice};

pub fn gmain() {
    run_state(NullState, &STATES_FROM_NULL_STATE, &FNS_ON_NULL_STATE);
}

fn run_state<B, S>(base: B, sub_states: &[fn(S)], tests: &[fn(S)])
where
    S: StateFrom<B> + DevClient + Clone
{
    let state: S = S::state_from(base);
    let mut snap_id = state.snap();
    for runner in sub_states {
        runner(state.clone());
        state.reset(snap_id);
        snap_id = state.snap();
    }
    for t in tests {
        t(state.clone());
        state.reset(snap_id);
        snap_id = state.snap();
    }
}

pub trait StateFrom<T> { fn state_from(src: T) -> Self; }
impl<T> StateFrom<T> for T { fn state_from(src: T) -> Self { src }}


#[derive(Debug, Clone)]
pub struct NullState;
impl DevClient for NullState {}
#[distributed_slice]
pub static FNS_ON_NULL_STATE: [fn(NullState)] = [..];
#[distributed_slice]
pub static STATES_FROM_NULL_STATE: [fn(NullState)] = [..];

pub trait DevClient {
    // client() needs to be impl by users so we can provide default
    // impls for snap and reset
    // fn client(&self) -> DevRpcMiddleware;
    fn snap(&self) -> u64 { 17 }
    fn reset(&self, id: u64) {}
}

#[derive(Debug, Clone)]
pub struct S1;
impl StateFrom<NullState> for S1 { fn state_from(s: NullState) -> Self { S1 } }

// macro generated:
impl DevClient for S1 {}
#[distributed_slice]
pub static FNS_ON_S1: [fn(S1)] = [..];
#[distributed_slice]
pub static STATES_FROM_S1: [fn(S1)] = [..];
#[distributed_slice(STATES_FROM_NULL_STATE)]
static _S01: fn(NullState) = run_s1;
fn run_s1(b: NullState) { run_state(b, &STATES_FROM_S1, &FNS_ON_S1) }


#[derive(Debug, Clone)]
pub struct S2;
impl StateFrom<S1> for S2 { fn state_from(s: S1) -> Self { S2 } }

impl DevClient for S2 {}
#[distributed_slice]
pub static FNS_ON_S2: [fn(S2)] = [..];
#[distributed_slice]
pub static STATES_FROM_S2: [fn(S2)] = [..];
#[distributed_slice(STATES_FROM_S1)]
static _S11: fn(S1) = run_s2;
fn run_s2(b: S1) { run_state(b, &STATES_FROM_S2, &FNS_ON_S2) }

#[distributed_slice(FNS_ON_S1)]
static _T11: fn(S1) = act_on_s1a;
pub fn act_on_s1a(_: S1) { println!("act_on_s1a"); }
#[distributed_slice(FNS_ON_S1)]
static _T12: fn(S1) = act_on_s1b;
pub fn act_on_s1b(_: S1) { println!("act_on_s1b"); }
#[distributed_slice(FNS_ON_S2)]
static _T21: fn(S2) = act_on_s2a;
pub fn act_on_s2a(_: S2) { println!("act_on_s2a"); }
#[distributed_slice(FNS_ON_S2)]
static _T22: fn(S2) = act_on_s2b;
pub fn act_on_s2b(_: S2) { println!("act_on_s2b"); }


// would need to take the array of child states and
// fn run_<S: From<B>, B>(base: B) {
//     let state: S = S::from(base);
//     for run_sub_state in STATES_FROM_NULL {
//         run_sub_state(state.clone())
//         state.reset();
//     }
//     for f in FNS_ON_NULL_STATE
// }
// fn run_null(base: NullState) {
//     let state: NullState = NullState::state_from(base); // must store snap_id
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
// fn run_s1(base: NullState) {
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