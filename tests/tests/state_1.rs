use crate::tests::state_0::{State0, STATES_FROM_STATE0};
use crate::tests::utils::*;
use ethers::{prelude::LocalWallet};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct State1 {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
    pub machine: SimpleState<Client>, // deployed SimpleState contract
}

#[async_trait]
impl State for State1 {
    type Base = State0;

    // deploy the SimpleState contract
    async fn new(base: Self::Base) -> Result<Self> {

        // step contract to state 1
        base.machine.step(1.into()).send().await?;

        Ok(Self {
            client: base.client,
            accts: base.accts,
            machine: base.machine,
        })
    }
}

pub async fn test_step_to_1(ctx: State1) -> Result<()> {
    let machine_state = ctx.machine.state().call().await?;
    assert_eq!(machine_state, 1.into());
    Ok(())
}

pub async fn test_step_to_3(ctx: State1) -> Result<()> {
    ctx.machine.step(3.into()).send().await?;
    let machine_state = ctx.machine.state().call().await?;
    assert_eq!(machine_state, 3.into());
    Ok(())
}

// --- macro generated ---

impl TestSet for State1 {
    type State = State1;
    type Runner = RunnerType;
    fn tests(&self) -> &'static [Test<Self::State>] {
        &TESTS_ON_STATE1
    }
    fn children(&self) -> &'static [StateMove<Self::State, Self::Runner>] {
        &STATES_FROM_STATE1
    }
}

#[distributed_slice]
pub static STATES_FROM_STATE1: [StateMove<State1, RunnerType>] = [..];
#[distributed_slice]
pub static TESTS_ON_STATE1: [Test<State1>] = [..];
#[distributed_slice(TESTS_ON_STATE1)]
pub static __TS11: Test<State1> = Test {
    name: "test_step_to_1",
    run: |s| Box::pin(test_step_to_1(s)),
};
#[distributed_slice(TESTS_ON_STATE1)]
pub static __TS12: Test<State1> = Test {
    name: "test_step_to_1",
    run: |s| Box::pin(test_step_to_3(s)),
};

#[distributed_slice(STATES_FROM_STATE0)]
pub static __SN1: StateMove<State0, RunnerType> =
    |s, r| Box::pin(dispatch::<State0, State1, RunnerType>(s, r));
