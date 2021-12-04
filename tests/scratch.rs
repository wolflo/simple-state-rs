use anyhow::{anyhow, Result};
use async_trait::async_trait;
use linkme::{distributed_slice, DistributedSlice};

use crate::types::Action;

pub trait BuildFrom<T> {}

pub trait TestSet {
    type Base: Ctx;
    fn ctx(&self) -> Self::Base;
    fn tests(&self) -> DistributedSlice<[Test<Self::Base>]>;
    // fn next(&self) -> Option<&'static dyn TestSet<Base=&'static dyn BuildFrom<Self::Base>>>;
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
pub struct Ctx1();
#[async_trait]
impl Ctx for Ctx1 {
    type Base = NullCtx;
    async fn reset(&self) {}
    async fn build(&self, b: Self::Base) {}
}
pub struct Test<T> {
    pub name: &'static str,
    pub run: Action<T>
}
pub struct Tset<T> {
    pub tests: DistributedSlice<[Test<T>]>,
}
impl<T: Ctx> TestSet for Tset<T> {
    type Base = T;
    fn ctx(&self) -> Self::Base { unreachable!() }
    fn tests(&self) -> DistributedSlice<[Test<Self::Base>]> { self.tests }
    // fn next(&self) -> Option<&'static dyn TestSet<Base=&'static dyn BuildFrom<Self::Base>>> { None }
}

#[distributed_slice]
pub static SETS1: [&'static (dyn TestSet<Base=Ctx1> + Send + Sync)] = [..];
// #[distributed_slice(SETS1)]
// static _S1: &'static (dyn TestSet<Base=Ctx1> + Send + Sync) = &Tset { tests: T11 };

// #[distributed_slice]
// pub static T11: [Test<Ctx1>] = [..];
// #[distributed_slice(T11)]
// static _T11: Test<Ctx1> = Test { name: "t11", run: |x| Box::pin(t11(x)) };
// #[distributed_slice(T11)]
// static _T12: Test<Ctx1> = Test { name: "t12", run: |x| Box::pin(t12(x)) };


// async fn t11(ctx: Ctx1) -> Result<()> { println!("running t11"); Ok(()) }
// async fn t12(ctx: Ctx1) -> Result<()> { println!("running t12"); Ok(()) }
