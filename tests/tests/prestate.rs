use crate::harness::types::{DevRpcInitState, TESTS_ON_INIT_STATE};
use crate::tests::utils::*;

async fn test_initialization(state: DevRpcInitState) -> Result<()> {
    let block_number = state.client.get_block_number().await?;
    let balance = state
        .client
        .get_balance(state.accts[0].address(), None)
        .await?;
    assert_eq!(block_number, 0usize.into());
    assert!(balance > 0usize.into());
    Ok(())
}

#[distributed_slice(TESTS_ON_INIT_STATE)]
pub static __ST1: Test<DevRpcInitState> = Test {
    name: "test_initialization",
    run: |s| Box::pin(test_initialization(s)),
};
