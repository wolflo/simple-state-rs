#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::future::Future;
use linkme::{distributed_slice, DistributedSlice};

pub type AsyncResult = std::pin::Pin<Box<dyn Future<Output = Result<()>>>>;
pub type Action<T> = fn(&'static T) -> AsyncResult;

pub struct Test<T: 'static> {
    pub name: &'static str,
    pub run: Action<T>,
}

pub struct Ctx0();
pub struct Ctx1();
pub struct Ctx2();
pub enum FromCNull {
    C0(Ctx0),
}
pub enum FromC0 {
    C1(Ctx1),
    C2(Ctx2),
}

// ---
#[distributed_slice]
pub static LL: [DistributedSlice<[Test<Ctx0>]>] = [..];
// #[distributed_slice(LL)]
// static _LL0: DistributedSlice<[Test<]

// --- Test lists ---
#[distributed_slice]
pub static TLIST0: [Test<FromCNull>] = [..]; // Only going to have 1 variant of each in here anyway
                                             // #[distributed_slice(TLIST0)]
                                             // static _TL01: Test<Ctx0> = Test { name: "tl01", run: |x| Box::pin(tl01(x)) };
                                             // #[distributed_slice(TLIST0)]
                                             // static _TL02: Test<Ctx0> = Test { name: "tl02", run: |x| Box::pin(tl02(x)) };
                                             // #[distributed_slice]
                                             // pub static TLIST1: [Test<Ctx1>] = [..];
                                             // #[distributed_slice(TLIST1)]
                                             // static _TL11: Test<Ctx1> = Test { name: "tl11", run: |x| Box::pin(tl11(x)) };
                                             // #[distributed_slice(TLIST1)]
                                             // static _TL12: Test<Ctx1> = Test { name: "tl12", run: |x| Box::pin(tl12(x)) };

async fn tl01(ctx: &Ctx0) -> Result<()> {
    println!("running tl01");
    Ok(())
}
async fn tl02(ctx: &Ctx0) -> Result<()> {
    println!("running tl02");
    Ok(())
}
async fn tl11(ctx: &Ctx1) -> Result<()> {
    println!("running tl11");
    Ok(())
}
async fn tl12(ctx: &Ctx1) -> Result<()> {
    println!("running tl12");
    Ok(())
}

// // need an iterator over Tsets, where next() is stateful and returns a
// // Tset of a different type. This means our type Item is different each time.
// trait Context {
//     fn reset(b built from Self) -> Self
// }
// type Test {
//     name;
//     fn act(Context)
// }
// trait State {
//     type Item: State<;
//     fn next(self, fn(Self::Item) -> NextCtx) -> Option<NextCtx>;
// }

// trait State<A> {
//     fn next(State<A>, fn(A) -> B) -> State<B>;
// }

// fn iter_tests([ctxs; N], [[fn(ctx); M]; N])

// trait CtxIterator {
//     type Ctx<T>;
//     fn next(self) -> CtxIterator<Ctx: BuildFrom<Self::Item>> ?

//     // could have two next() methods, one that moves to the next buildable
//     // Ctx until exhausted, then the other moves to the next Ctx at the same
//     // level
//     // Ctx
// }

// fn run_tests(Ctx, [fn(Ctx); const N])

// trait Context { fn reset(); }
// trait Foo {
//     type Assoc where Self: Context;
// }

// // I can have separate lists of "tests over C" for every C
// // The problem is how to iterate over the C's
// trait Ctx {
//     type Prev;
//     fn reset(&mut self); // tests can mutate ctx, but we save old version of it
//     // fn sibling(&self) -> dyn Ctx<Prev=Self::Prev>; // Ctx build from same parent
//     fn child<T>(&self) -> T where T: Ctx;
// }

// // could every ctx have an associated next ctx? If so, we
// // don't need to state "next can be generated from us". Because
// // there is only one next state that can be generated from us

// // **
// // associated_type_bounds allows:
// // fn fizzbuzz() -> impl Iterator<Item = String>; to be:
// // fn fizzbuzz() -> impl Iterator<Item: Display>
// // fn_traits

// fn next() -> TestSet<AssociatedCtx: From<Self::Ctx>>

// next() -> Iterator<Item: TestSet<CtX>>

// // * the values contained in a context are separete from it's chain state.
// // The chain state only requires that it be differentiated from other chain states.
// // It's not traits we need to avoid, but trait objects
// type TestState {
//     state: chain state
//     ctx: values related to the chain state
//     tests: a list of tests runnable against current state, ctx
//     children: a list of TestStates that can be generated from current TestState

//     build(): a method to build state and ctx from parent
// }

// distributed slices we need:
// for all (ctx, state): [fn(ctx, state)] -- Tests
// for all TestState: [TestState]         -- Children

// [TestState<AssocType: BuildFrom<Self>>]

// trait ctx
// struct TestSet {
//     ctx:
// }

// trait Ctx {
//     fn run_tests(tests: [Tests that act on me]) {
//         for t in tests { t.run; self.reset(); }
//     }
// }

// each state should have a `struct State9` which differentiates it from other states at the type level.

// trait Test {
//     type Ctx<T>;
//     fn run(&mut self) -> Result<()>;
// }

// // First step should be to make a slice of functions acting on different contexts
// // i.e. need [fn] rather than [fn(type)]
// // type constructor is Vec

// struct Foo<X: DomainFamily> {
//     bar: X::
// }
// // https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md
// // want to be able to make a list of *Types*
// // ** not a thing of the type, but the type itself
// // give me a type, where that type implements Ctx
// // Rc example says give me type, where that type implements Deref
// // Same way Foo takes either Arc or Rc, my fns should take either Ctx1 or Ctx2

// // Let's make a list of functions that take either an Arc<u8> OR a Rc<u8>

use std::{ops::Deref, rc::Rc, sync::Arc};

trait PointerFamily {
    type Pointer<T>: Deref<Target = T>;
    fn new<T>(value: T) -> Self::Pointer<T>;
}

struct ArcFamily;

impl PointerFamily for ArcFamily {
    type Pointer<T> = Arc<T>;
    fn new<T>(value: T) -> Self::Pointer<T> {
        Arc::new(value)
    }
}

struct RcFamily;

impl PointerFamily for RcFamily {
    type Pointer<T> = Rc<T>;
    fn new<T>(value: T) -> Self::Pointer<T> {
        Rc::new(value)
    }
}

struct Foo<P: PointerFamily> {
    bar: P::Pointer<String>, // bar: Arc<String> | Rc<String>
}
struct Baz<P: Deref<Target = String>> {
    bar: P,
}

struct MyList<P: PointerFamily> {
    pointer: P::Pointer<String>,
}
fn take_ptrs<P: PointerFamily>(l: [P; 2]) {}
fn call_ptrs() {
    // let z = [Arc::new(1), Arc::new(2)];
    // take_ptrs(z);
}

// fn make_lst_o_ptrs<T>() {
//     let l: [PointerFamily::Pointer<T>; 2] = [Arc::new(1), Arc::new(2)];
// }

fn foo() {}
fn bar() {}
fn baz(a: usize) {}
fn zoo(b: u32) {}
fn take_arc_type<P: PointerFamily<Pointer<u8> = Arc<u8>>>(a: u8) {
    let bar: Arc<u8> = P::new(a);
}
fn take_rc_type<P: PointerFamily<Pointer<u8> = Rc<u8>>>(a: u8) {
    let bar: Rc<u8> = P::new(a);
}
//@wol***
// fn take_ctx1_type::<Ctx1Family>(b: BaseContext)
fn take_arg_as_rc_type<P: PointerFamily<Pointer<u8> = Rc<u8>>>(b: P::Pointer<u8>) {
    let c: Rc<u8> = b;
}
fn take_arg_as_arc_type<P: PointerFamily<Pointer<u8> = Arc<u8>>>(b: P::Pointer<u8>) {
    let c: Arc<u8> = b;
}
fn take_any_ptr<P: PointerFamily>() {}
fn make_lst() {
    let l = [foo, bar];
    let z = [take_arc_type::<ArcFamily>, take_rc_type::<RcFamily>];
    let y = [take_any_ptr::<ArcFamily>, take_any_ptr::<RcFamily>];
    // let z = [take_arg_as_arc_type::<ArcFamily>, take_arg_as_rc_type::<RcFamily>];
}

trait StateFam {
    type State<T>;
}
struct S1<T>(T);
struct S2<T>(T);
struct S1Fam;
struct S2Fam;
impl StateFam for S1Fam {
    type State<U> = S1<U>;
}
impl StateFam for S2Fam {
    type State<U> = S2<U>;
}
fn take_s1<P: StateFam<State<u8> = S1<u8>>>() {}
fn take_s2<P: StateFam<State<u8> = S2<u8>>>() {}
fn bbbb() {
    // let z = [take_s1::<S1Fam>, take_s2::<S2Fam>];
    let z: [RcFamily; 2];
}
