use crate::tests::utils::*;
use std::sync::Arc;
use ethers::prelude::LocalWallet;
use crate::harness::types::{DevRpcInitState, STATES_FROM_INIT_STATE};

#[derive(Debug, Clone)]
pub struct State0 {
    pub client: Arc<Client>,
    pub accts: Vec<LocalWallet>,
}
#[async_trait]
impl State for State0 {
    type Prev = DevRpcInitState;
    async fn new(prev: Self::Prev) -> Result<Self> {
        println!("Building State0");
        Ok(Self { client: prev.client, accts: prev.accts })
    }
}

pub async fn test_deploy(state: State0) -> Result<()> {
    Ok(())
}

impl TestSet for State0 {
    type State = State0;
    type Runner = RunnerType;
}

#[distributed_slice(STATES_FROM_INIT_STATE)]
pub static __SN1: StateMove<DevRpcInitState, RunnerType> =
    |s, r| Box::pin(dispatch::<DevRpcInitState, State0, RunnerType>(s, r));
