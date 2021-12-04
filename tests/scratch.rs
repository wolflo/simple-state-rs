use anyhow::{anyhow, Result};
use futures::future::Future;
use async_trait::async_trait;
use linkme::{distributed_slice, DistributedSlice};

pub type AsyncResult = std::pin::Pin<Box<dyn Future<Output = Result<()>>>>;
pub type Action<T> = fn(&'static T) -> AsyncResult;

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
pub struct Test<T: ?Sized + 'static> {
    pub name: &'static str,
    pub run: Action<T>
}
pub struct Tset<'a, T: ?Sized + 'static, U: BuildFrom<T>> {
    pub ctx: &'a T,
    pub tests: DistributedSlice<[Test<T>]>,
    pub next: Option<&'a (dyn TestSet<'a, &'a dyn Is<T, U>> + Send + Sync)>,
}
// pub struct Tset<'a, T: ?Sized + 'static> {
//     pub ctx: &'a T,
//     pub tests: DistributedSlice<[Test<T>]>,
//     pub next: Option<&'a (dyn TestSet<'a, &'a dyn BuildFrom<T>> + Send + Sync)>,
// }
// T: &dyn BuildFrom<Ctx0>
// impl for Tset<_, &dyn Is<Ctx0, Ctx1>>
// https://users.rust-lang.org/t/transitive-traits/47106 suggests blanket impl will
// work, as long as each Ctx can be generated from only one predecessor
// need to say BuildFrom<0> == 1. TestSet<_, BuildFrom<0>> should match Tset<_, Is<0, 1>>
// or that 
// #[async_trait]
// impl BuildFrom<Ctx0> for &'_ dyn Is<Ctx0, Ctx1> { async fn build_from(&mut self, t: Ctx0) { unreachable!() }}
impl<'a, T: Ctx, U: BuildFrom<T>> TestSet<'a, T> for Tset<'a, T, U> {
    fn ctx(&self) -> &T { &self.ctx }
    fn tests(&self) -> DistributedSlice<[Test<T>]> { self.tests }
    fn next(&'a self) -> Option<&(dyn TestSet<&dyn BuildFrom<T>> + Send + Sync)> { self.next }
}
// impl<'a, T: BuildFrom<Ctx0>, U: Is<Ctx0, Ctx1>> TestSet<'a, Ctx0> for Tset<'a, Ctx1> {
//     fn ctx(&self) -> &Ctx1 { &self.ctx }
//     fn tests(&self) -> DistributedSlice<[Test<Ctx1>]> { self.tests }
//     fn next(&'a self) -> Option<&(dyn TestSet<&dyn BuildFrom<Ctx1>> + Send + Sync)> { self.next }
// }
// impl<'a, T: Ctx, U: BuildFrom<T>> TestSet<'a, T> for Tset<'a, U> {
//     fn ctx(&self) -> &T { &self.ctx }
//     fn tests(&self) -> DistributedSlice<[Test<T>]> { self.tests }
//     fn next(&'a self) -> Option<&(dyn TestSet<&dyn BuildFrom<T>> + Send + Sync)> { self.next }
// }

#[distributed_slice]
pub static TSETS: [&'static (dyn TestSet<'static, Ctx0> + Send + Sync)] = [..];
// #[distributed_slice(TSETS)]
// static _TS0: &'static (dyn TestSet<'static, Ctx0> + Send + Sync) = &Tset { ctx: &Ctx0(), tests: TLIST0, next: Some(_TS1) };

// static _TS1: &'static Tset<&dyn Is<Ctx0, Ctx1>> = &Tset { ctx: &(&Ctx1() as &'static dyn Is<Ctx0, Ctx1>), tests: TLIST1, next: None };
// static _TS1: &'static Tset<&dyn BuildFrom<Ctx0>> = &Tset { ctx: &(&Ctx1() as &'static dyn BuildFrom<Ctx0>), tests: TLIST1, next: None };

#[distributed_slice]
pub static TLIST0: [Test<Ctx0>] = [..];
#[distributed_slice(TLIST0)]
static _TL01: Test<Ctx0> = Test { name: "tl01", run: |x| Box::pin(tl01(x)) };
#[distributed_slice(TLIST0)]
static _TL02: Test<Ctx0> = Test { name: "tl02", run: |x| Box::pin(tl02(x)) };
#[distributed_slice]
pub static TLIST1: [Test<&'static dyn Is<Ctx0, Ctx1>>] = [..];
#[distributed_slice(TLIST1)]
static _TL11: Test<&'static dyn Is<Ctx0, Ctx1>> = Test { name: "tl11", run: |x| Box::pin(tl11(*x)) };
#[distributed_slice(TLIST1)]
static _TL12: Test<&'static dyn Is<Ctx0, Ctx1>> = Test { name: "tl12", run: |x| Box::pin(tl12(*x)) };

// #[async_trait]
// impl<T: Send, U> BuildFrom<T> for &dyn Is<T, U> { async fn build_from(&mut self, t: T) { unreachable!() }}

// Super == BuildFrom<0>
// Sub   == Is<0, 1>
trait AsBase<T> {
    // self: dyn Is<X, _> -> dyn BuildFrom<X>
    fn as_base(&self) -> &dyn BuildFrom<T>;
}
impl<T: Send, U> AsBase<T> for &dyn Is<T, U> {
    fn as_base(&self) -> &dyn BuildFrom<T> { *self }
}
// #[async_trait]
// pub trait BuildFrom<T: ?Sized>: AsDynSuper { async fn build_from(&mut self, t: T); }
#[async_trait]
pub trait BuildFrom<T: ?Sized> { async fn build_from(&mut self, t: T); }
pub trait IsSame<T> {}
impl IsSame<Ctx1> for Ctx1 {}
impl Is<Ctx0, Ctx1> for Ctx1 {}
pub trait Is<T, U>: BuildFrom<T> + IsSame<U> + Send + Sync {}

// const fn constrain<F, T: 'static>(f: F) -> F where F: Fn(&'static T) { f }
async fn tl01(ctx: &Ctx0) -> Result<()> { println!("running tl01"); Ok(()) }
async fn tl02(ctx: &Ctx0) -> Result<()> { println!("running tl02"); Ok(()) }

// // needs to accept any dyn BuildFrom<Ctx0> (as long as it is also Ctx1?)
async fn tl11<T: ?Sized>(ctx: &T) -> Result<()> where T: Is<Ctx0, Ctx1> { println!("running tl11"); Ok(()) }
async fn tl12<T: ?Sized>(ctx: &T) -> Result<()> where T: Is<Ctx0, Ctx1> { println!("running tl12"); Ok(()) }
// async fn tl12(ctx: Ctx1) -> Result<()> { println!("running tl12"); Ok(()) }
