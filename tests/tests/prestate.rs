use crate::harness::types::DevRpcInitState;
use crate::tests::utils::*;

#[ethtest]
async fn test_initialization(ctx: DevRpcInitState) -> Result<()> {
    let block_number = ctx.client.get_block_number().await?;
    let balance = ctx.client.get_balance(ctx.accts[0].address(), None).await?;
    assert_eq!(block_number, 0usize.into());
    assert!(balance > 0usize.into());
    Ok(())
}

// #[distributed_slice(TESTS_ON_INIT_STATE)]
// pub static __ST1: Test<DevRpcInitState> = Test {
//     name: "test_initialization",
//     run: |s| Box::pin(test_initialization(s)),
// };
