use crate::harness::types::DevRpcInitState;
use crate::tests::utils::*;
use ethers::prelude::LocalWallet;
use std::sync::Arc;

// Define the initial test state
#[derive(Debug, Clone)]
pub struct State0 {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
    pub machine: SimpleState<Client>, // deployed SimpleState contract
}

// Create the initial test state from the default state
#[ethstate]
#[async_trait]
impl State for State0 {
    type Base = DevRpcInitState;

    // deploy the SimpleState contract and add to State0
    async fn new(base: Self::Base) -> Result<Self> {
        let factory = make_factory("SimpleState", &base.client)?;
        let deployed = factory.deploy(())?.send().await?;
        let machine = SimpleState::new(deployed.address(), base.client.clone());
        Ok(Self {
            client: base.client,
            accts: base.accts,
            machine,
        })
    }
}

// test that the deployment in the state new() method was successful
#[ethtest]
async fn test_deploy(ctx: State0) -> Result<()> {
    let initial_state = ctx.machine.state().call().await?;
    assert_eq!(initial_state, 0.into());
    Ok(())
}

// --- macro generated ---
// impl TestSet for State0 {
//     type State = State0;
//     type Runner = RunnerType;
//     fn tests(&self) -> &'static [Test<Self::State>] {
//         &TESTS_ON_STATE0
//     }
//     fn children(&self) -> &'static [StateMove<Self::State, Self::Runner>] {
//         &STATES_FROM_STATE0
//     }
// }

// #[distributed_slice]
// pub static STATES_FROM_STATE0: [StateMove<State0, RunnerType>] = [..];

// #[distributed_slice]
// pub static TESTS_ON_STATE0: [Test<State0>] = [..];
// #[distributed_slice(TESTS_ON_STATE0)]
// pub static __TS01: Test<State0> = Test {
//     name: "test_deploy",
//     run: |s| Box::pin(test_deploy(s)),
// };

// #[distributed_slice(STATES_FROM_INIT_STATE)]
// pub static __SN1: StateMove<DevRpcInitState, RunnerType> =
//     |s, r| Box::pin(dispatch::<DevRpcInitState, State0, RunnerType>(s, r));
