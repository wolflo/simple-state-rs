use anyhow::{anyhow, Result};
use async_trait::async_trait;
use linkme::{distributed_slice, DistributedSlice};

use crate::types::Action;

#[async_trait]
pub trait BuildFrom<T> { async fn build_from(&mut self, t: T); }
pub trait TestSet<T> {
    fn ctx(&mut self) -> &mut T;
    fn tests(&self) -> DistributedSlice<[Test<T>]>;
    fn next(&self) -> Option<&'static dyn TestSet<&'static dyn BuildFrom<T>>>;
}
#[async_trait]
pub trait Ctx {
    type Base: Ctx;
    async fn reset(&self);
    async fn build(&self, b: Self::Base);
}

pub struct NullCtx();
#[async_trait]
impl Ctx for NullCtx {
    type Base = NullCtx;
    async fn reset(&self) { () }
    async fn build(&self, b: Self::Base) { () }
}
pub struct Ctx0();
#[async_trait]
impl Ctx for Ctx0 {
    type Base = NullCtx;
    async fn reset(&self) {}
    async fn build(&self, b: Self::Base) {}
}
#[async_trait]
impl BuildFrom<NullCtx> for Ctx0 {
    async fn build_from(&mut self, t: NullCtx) { }
}
pub struct Test<T> {
    pub name: &'static str,
    pub run: Action<T>
}
pub struct Tset<T> {
    pub ctx: T,
    pub tests: DistributedSlice<[Test<T>]>,
}
impl<T: Ctx> TestSet<T> for Tset<T> {
    fn ctx(&mut self) -> &mut T { &mut self.ctx }
    fn tests(&self) -> DistributedSlice<[Test<T>]> { self.tests }
    fn next(&self) -> Option<&'static dyn TestSet<&'static dyn BuildFrom<T>>> { None }
}

#[distributed_slice]
pub static TSETS: [&'static (dyn TestSet<Ctx0> + Send + Sync)] = [..];
#[distributed_slice(TSETS)]
static _TS0: &'static (dyn TestSet<Ctx0> + Send + Sync) = &Tset { ctx: Ctx0(), tests: TLIST0 };

#[distributed_slice]
pub static TLIST0: [Test<Ctx0>] = [..];
#[distributed_slice(TLIST0)]
static _TL01: Test<Ctx0> = Test { name: "tl01", run: |x| Box::pin(tl01(x)) };
#[distributed_slice(TLIST0)]
static _TL02: Test<Ctx0> = Test { name: "tl02", run: |x| Box::pin(tl02(x)) };


async fn tl01(ctx: Ctx0) -> Result<()> { println!("running tl01"); Ok(()) }
async fn tl02(ctx: Ctx0) -> Result<()> { println!("running tl02"); Ok(()) }
