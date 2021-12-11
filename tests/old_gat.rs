#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use linkme::{distributed_slice, DistributedSlice};

pub struct C1;
pub struct C2;
pub fn act_on_c1a(_: C1) {}
pub fn act_on_c1b(_: C1) {}
pub fn act_on_c2a(_: C2) {}
pub fn act_on_c2b(_: C2) {}

trait Context<S> { fn reset(); }
trait CtxFam {
    type Ctx<S>: Context<S>;
}

// Want to make a list of functions polymorphic over Contexts
// Option is Ctx
// We have two options actually. We can say that all Ctxs are part of
// one overarching type, and we have a list of functions that act on
// that overarching type (even though these functions are partial over
// that type). Alternatively, we could say that all functions belong
// to an overarching type, and we have a list of this type?
// Probably makes more sense to say that all TestStates belong to an
// overarching type, and we have a list of items of this type

struct Block<S> { state: S, }

// [ Block<S1> | Block<S2> ]
// [ Block< S1 | S2 >]

trait FnFam {
    type F<R>: Fn();
}

trait StateFamily {
    type B<S>;
    fn new<S>(state: S);
}

// trait FnFam {
//     // We want to make a list of Fn
//     // so "Func" is * -> *. When applied to a type T, it gives an Fn(T)
//     type Func<T>;
//     fn new<T>(arg: T) -> Self::Func<T>;
// }

// trait ArgFam {
//     type Arg<T>;
//     fn new<T>(x: T) -> Self::Arg<T>;
// }

// impl ArgFam for C1 {
//     type Arg<T> = C1<T>;
//     fn new<T>(x: T) -> Self::Pointer<T> {
//         C1;
//     }
// }

// pub fn act_on_c1a_gen<>(c: C::Ctx) { let x: C1 = c; }

#[distributed_slice]
pub static FNS_C1: [fn(C1)] = [..];
#[distributed_slice(FNS_C1)]
static _F11: fn(C1) = act_on_c1a;
#[distributed_slice(FNS_C1)]
static _F12: fn(C1) = act_on_c1b;

// I want TestBlock to contain a list of TestBlocks that are associated,
// but slightly different (i.e. they act over a state that can be built from our state)
// Need to be able to build a slice of test states that act on different states,
// but all act on a state that can be built from the same initial state
// For ex we use GATs bc we don't want to draw equivalence between Arc<u8> and Rc<u8>,
// but between Arc and Rc. In the same way, we dont want equivalence between
// Block<S1> and Block<S2>, but between Block and Block?

// Let's make a structure where we can have 2 linked state inputs, 1 of which is buildable from the other

// trait State {}
// trait BuildFrom<T> {}
// struct S1; impl State for S1 {}
// struct S2; impl State for S2 {} impl BuildFrom<S1> for S2 {}
// struct S3; impl State for S3 {} impl BuildFrom<S1> for S3 {}
// impl<T> BuildFrom<T> for T {}
// impl<T, U> State for T where T: BuildFrom<U> {}
// impl From<S1> for S2 { fn from(src: S1) -> Self {todo!()} }
// impl From<S1> for S3 { fn from(src: S1) -> Self {todo!()} }
// // struct StateNode<S: State, N: From<S> + 'static> {
// struct StateNode<S: State + 'static, N: State + BuildFrom<S>> {
//     state: S,
//     next: &'static StateNode<N, &'static dyn BuildFrom<N>>,
// }

pub fn main() {
    // let node = StateNode { state: S1, next: &[&S2, &S3] };
}

// Start with a way to enumerate test states that are all buildable from NullState
// Can we make these type constructors, that when applied to a NullState give
// us a state. e.g. we can be generic over StateBuilder's (State1, State2, etc.), 
// where State1<NullState> is a type

pub struct NullState;
pub struct S1; impl From<NullState> for S1 { fn from(s: NullState) -> Self { todo!() }}
pub struct S2; impl From<NullState> for S2 { fn from(s: NullState) -> Self { todo!() }}

// for each state:
// - a list of builder fns that take the current state
// - a list of test fns to run

// StateBuilder/Family would impl:
// from()
// sub_states() or run_sub_states()
// tests() or run_tests()

// For each new state, generate:
// list of tests on that state
// a single function, generic over the type of the generated state, that takes this state as a BaseState arg and inits,runs sub tests, then runs tests
// a list of calls to that fn for each child state

// wait, could I do this without GATs?
// 1 advantage of gats is that state S1 controls the running of it's child
// state tests and can impl reset
fn test_s1(base: NullState) {
    let state: S1 = base.into();
    for sub_state in STATES_FROM_S1 {
        sub_state(state.clone());
    }
    for t in TESTS_S1 {
        t(state.clone())
        state.reset();
    }
    // prev runner will reset anyway if we don't run any tests
}
