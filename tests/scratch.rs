use anyhow::{anyhow, Result};
use futures::future::Future;
use async_trait::async_trait;
use linkme::{distributed_slice, DistributedSlice};
use downcast_rs::{Downcast, impl_downcast};
use std::any::Any;

pub type AsyncResult = std::pin::Pin<Box<dyn Future<Output = Result<()>>>>;
pub type Action<T> = fn(&'static T) -> AsyncResult;

pub enum FromCNull { C0(Ctx0) }
pub enum FromC0 { C1(Ctx1) }

pub trait TestSet<'a, T: ?Sized> {
    fn ctx(&self) -> &T;
    fn tests(&self) -> DistributedSlice<[Test<T>]>;
    fn next(&'a self) -> Option<&(dyn TestSet<&dyn BuildFrom<T>> + Send + Sync)>;
    // fn next(&self) -> Option<&'static (dyn TestSet<&'static dyn BuildFrom<T>> + Send + Sync)>;
}
#[async_trait]
pub trait Ctx {
    type Base: Ctx;
    async fn reset(&mut self);
    async fn build(&mut self, b: Self::Base);
}

pub struct NullCtx();
#[async_trait]
impl Ctx for NullCtx {
    type Base = NullCtx;
    async fn reset(&mut self) { () }
    async fn build(&mut self, b: Self::Base) { () }
}
pub struct Ctx0();
pub struct Ctx1();
#[async_trait]
impl Ctx for Ctx0 {
    type Base = NullCtx;
    async fn reset(&mut self) {}
    async fn build(&mut self, b: Self::Base) {}
}
#[async_trait]
impl Ctx for Ctx1 {
    type Base = Ctx0;
    async fn reset(&mut self) {}
    async fn build(&mut self, b: Self::Base) {}
}
#[async_trait]
impl BuildFrom<Ctx0> for Ctx1 {
    async fn build_from(&mut self, b: Ctx0) { self.build(b).await }
}
// #[async_trait]
// impl BuildFrom<&&dyn Is<Ctx0, Ctx1>> for &Ctx1 {
//     async fn build_from(&mut self, b: &&dyn Is<Ctx0, Ctx1>) { unreachable!() }
// }
// #[async_trait]
// impl<T: Send, U> BuildFrom<dyn Is<U, T>> for T
// {
//     async fn build_from(&mut self, b: dyn Is<U, T>) { unimplemented!() }
// }
pub struct Test<T: ?Sized + 'static> {
    pub name: &'static str,
    pub run: Action<T>
}
pub struct Tset<'a, T: ?Sized + 'static> {
    pub ctx: &'a T,
    pub tests: DistributedSlice<[Test<T>]>,
    pub next: Option<&'a (dyn TestSet<'a, &'a dyn BuildFrom<T>> + Send + Sync)>,
}
// T: &dyn BuildFrom<Ctx0>
// impl for Tset<_, &dyn Is<Ctx0, Ctx1>>
// https://users.rust-lang.org/t/transitive-traits/47106 suggests blanket impl will
// work, as long as each Ctx can be generated from only one predecessor
// need to say BuildFrom<0> == 1. TestSet<_, BuildFrom<0>> should match Tset<_, Is<0, 1>>
// or that
// #[async_trait]
// impl BuildFrom<Ctx0> for &'_ dyn Is<Ctx0, Ctx1> { async fn build_from(&mut self, t: Ctx0) { unreachable!() }}
impl<'a, T: Ctx> TestSet<'a, T> for Tset<'a, T> {
    fn ctx(&self) -> &T { &self.ctx }
    fn tests(&self) -> DistributedSlice<[Test<T>]> { self.tests }
    fn next(&'a self) -> Option<&(dyn TestSet<&dyn BuildFrom<T>> + Send + Sync)> { self.next }
}

// dyn Is<C0, C1> <-> dyn BuildFrom<C0>
// need to prove that Any Tset<T> impls TestSet<U> as long as there is a path of BuildFrom's from U -> T
#[distributed_slice]
pub static TSETS: [&'static (dyn TestSet<'static, Ctx0> + Send + Sync)] = [..];
// #[distributed_slice(TSETS)]
// static _TS0: &'static (dyn TestSet<'static, Ctx0> + Send + Sync) = &Tset { ctx: &Ctx0(), tests: TLIST0, next: Some(_TS1) };

// static _TSX: &'static Tset<&dyn BuildFrom<Ctx0>> = &Tset { ctx: &(_TS1.ctx as &dyn BuildFrom<Ctx0>), tests: _TS1.tests as DistributedSlice<[Test<&dyn BuildFrom<Ctx0>>]>, next: None};
// static _TS1: &'static Tset<&dyn Is<Ctx0, Ctx1>> = &Tset { ctx: &(&Ctx1() as &'static dyn Is<Ctx0, Ctx1>), tests: TLIST1, next: None };
// static __TList1: DistributedSlice<[Test<&dyn BuildFrom<Ctx0>>]> = *TLIST1.static_slice();
// static __TList1: &'static [Test<&dyn BuildFrom<Ctx0>>] = TLIST1.static_slice() as &[Test<&'static (dyn BuildFrom<Ctx0>)>];
// static __TList1: &'static [Test<&dyn BuildFrom<Ctx0>>] = TLIST1.into_iter().map(|x| x.as_base()).collect();
// static __TList1: &'static [Test<&dyn BuildFrom<Ctx0>>] = &TLIST1.into_iter().map(|x| x.as_base()).collect::<Vec<_>>();
// static __TList1: &'static [Test<&dyn BuildFrom<Ctx0>>] = &TLIST1.into_iter().map(|x| x.as_base()).collect::<Vec<_>>();

use once_cell::sync::Lazy;
use array_map::*;
// static __TList1: Lazy<Vec<&Test<&dyn BuildFrom<Ctx0>>>> = Lazy::new(|| TLIST1.into_iter().map(|x| x.into_base()).collect::<Vec<_>>());

// can still try:
// - lazy collection of iter.map(conversion)
// -

// static _TS1: &'static Tset<&dyn BuildFrom<Ctx0>> = &Tset { ctx: &(&Ctx1() as &'static dyn BuildFrom<Ctx0>), tests: TLIST1, next: None };

// #[distributed_slice]
// pub static TLIST0: [Test<Ctx0>] = [..];
// #[distributed_slice(TLIST0)]
// static _TL01: Test<Ctx0> = Test { name: "tl01", run: |x| Box::pin(tl01(x)) };
// #[distributed_slice(TLIST0)]
// static _TL02: Test<Ctx0> = Test { name: "tl02", run: |x| Box::pin(tl02(x)) };
// #[distributed_slice]
// pub static TLIST1: [Test<&'static dyn Is<Ctx0, Ctx1>>] = [..];
// #[distributed_slice(TLIST1)]
// static _TL11: Test<&'static dyn Is<Ctx0, Ctx1>> = Test { name: "tl11", run: |x| Box::pin(tl11(*x)) };
// #[distributed_slice(TLIST1)]
// static _TL12: Test<&'static dyn Is<Ctx0, Ctx1>> = Test { name: "tl12", run: |x| Box::pin(tl12(*x)) };

#[distributed_slice]
pub static TLIST1: [Test<Ctx1>] = [..];
// #[distributed_slice(TLIST1)]
// static _TL11: Test<Ctx1> = Test { name: "tl11", run: |x| Box::pin(tl11(x)) };

// So I can coerce Ctx1 to BuildFrom<Ctx0> then pass this to a fn that's arg type is
// BuildFrom<Ctx0> but then downcasts it to Ctx1. But what I really want is to coerce
// a fn(Ctx1) to a fn(BuildFrom<Ctx0>) without changing the actual function arg type
// pub trait BF<T>: Downcast {}
// impl_downcast!(BF<T>);
// impl BF<Ctx0> for Ctx1 {}
// static __H: Action<dyn BuildFrom<Ctx0>> = |x| Box::pin(tl11(x));
// static __H: Action<dyn BF<Ctx0>> = |x| Box::pin(tl11(x));
// static __X: &dyn BuildFrom<Ctx0> = &Ctx1();
// static __F: fn(&dyn BuildFrom<Ctx0>) = |x| z(x);
// fn z(_: &Ctx1) {}
// pub static _TLC: Lazy<&[Test<dyn BuildFrom<Ctx0>>]> = Lazy::new(|| TLIST1.into_iter().map(|x| x.into()).collect::<Vec<_>>());
// pub static _TLC: Lazy<Vec<Test<dyn BuildFrom<Ctx0>>>> = Lazy::new(|| TLIST1.into_iter().map(|x| x.into()).collect::<Vec<_>>());
// impl From<&Test<Ctx1>> for Test<dyn BuildFrom<Ctx0>> {
//     fn from(x: &Test<Ctx1>) -> Self { x.into() }
// }
// https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions
// T to dyn U, when T implementes U + sized, and is object safe
// Ctx1 to dyn BuildFrom<Ctx0>
// impl From<&'static Ctx1> for &'static dyn BuildFrom<Ctx0> {
//     fn from(x: &'static Ctx1) -> Self { x }
// }
// impl<T, U: BuildFrom<T>> From<Test<U>> for Test<dyn BuildFrom<T>> {
//     fn from(x: Test<U>) -> Self { &x  as &Test<dyn BuildFrom<T>>}
// }


// any T can be built from itself
#[async_trait]
impl<T: Send + Sync> BuildFrom<T> for T { async fn build_from(&mut self, t: T) { *self = t } }

pub trait IntoBase<Base: ?Sized> { fn into_base(&self) -> &Base; }
impl<'a, T: 'a + BuildFrom<U>, U> IntoBase<dyn BuildFrom<U> + 'a> for T {
    fn into_base(&self) -> &(dyn BuildFrom<U> + 'a) { self }
}


#[async_trait]
pub trait BuildFrom<T: ?Sized>: Sync { async fn build_from(&mut self, t: T); }
pub trait IsSame<T> {}
impl IsSame<Ctx1> for Ctx1 {}
// impl Is<Ctx0, Ctx1> for Ctx1 {}
// pub trait Is<T, U>: BuildFrom<T> + IsSame<U> + Send + Sync {}

async fn tl01(ctx: &Ctx0) -> Result<()> { println!("running tl01"); Ok(()) }
async fn tl02(ctx: &Ctx0) -> Result<()> { println!("running tl02"); Ok(()) }

pub enum Ctxs { N(NullCtx), C0(Ctx0), C1(Ctx1), }

async fn run_enum(c: Ctxs) -> Result<()> {
    match c {
        Ctxs::C1(ctx) => tl11(ctx),
        _ => todo!(),
    }.await
}

async fn tl11(ctx: Ctx1) -> Result<()> { println!("running tl11"); Ok(()) }
// accepts any dyn BuildFrom<Ctx0> and downcasts to expected Ctx1
// async fn tl11<T: ?Sized + BuildFrom<Ctx0> + Any>(ctx_: &T) -> Result<()> {
//     let ctx = (ctx_ as &dyn Any).downcast_ref::<Ctx1>().expect("Bad downcast");
//     Ok(())
// }
// async fn tl11(ctx: &Ctx1) -> Result<()> { println!("running tl11"); Ok(()) }
// async fn tl12<T: ?Sized>(ctx: &T) -> Result<()> where T: Is<Ctx0, Ctx1> { println!("running tl12"); Ok(()) }
// async fn tl11<T: ?Sized>(ctx: &T) -> Result<()> where T: Is<Ctx0, Ctx1> { println!("running tl11"); Ok(()) }
// async fn tl12<T: ?Sized>(ctx: &T) -> Result<()> where T: Is<Ctx0, Ctx1> { println!("running tl12"); Ok(()) }
// async fn tl12(ctx: Ctx1) -> Result<()> { println!("running tl12"); Ok(()) }
