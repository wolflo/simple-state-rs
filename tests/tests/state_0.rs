use crate::harness::types::{DevRpcInitState, STATES_FROM_INIT_STATE};
use crate::tests::utils::*;
use ethers::prelude::LocalWallet;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct State0 {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
}
#[async_trait]
impl State for State0 {
    type Base = DevRpcInitState;
    async fn new(base: Self::Base) -> Result<Self> {
        println!("Building State0");
        Ok(Self {
            client: base.client,
            accts: base.accts,
        })
    }
}

pub async fn test_deploy(state: State0) -> Result<()> {
    // println!("test_deploy");
    Ok(())
}

impl TestSet for State0 {
    type State = State0;
    type Runner = RunnerType;
    fn tests(&self) -> &'static [Test<Self::State>] {
        &TESTS_ON_STATE0
    }
    fn children(&self) -> &'static [StateMove<Self::State, Self::Runner>] {
        &STATES_FROM_STATE0
    }
}

#[distributed_slice]
pub static STATES_FROM_STATE0: [StateMove<State0, RunnerType>] = [..];
#[distributed_slice]
pub static TESTS_ON_STATE0: [Test<State0>] = [..];
#[distributed_slice(TESTS_ON_STATE0)]
pub static __TS01: Test<State0> = Test {
    name: "test_deploy",
    run: |s| Box::pin(test_deploy(s)),
};

#[distributed_slice(STATES_FROM_INIT_STATE)]
pub static __SN1: StateMove<DevRpcInitState, RunnerType> =
    |s, r| Box::pin(dispatch::<DevRpcInitState, State0, RunnerType>(s, r));
