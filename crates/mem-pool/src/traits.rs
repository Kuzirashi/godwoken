use std::time::Duration;

use anyhow::Result;
use gw_types::{
    offchain::{CollectedCustodianCells, DepositInfo, RollupContext},
    packed::WithdrawalRequest,
};
use smol::Task;

pub trait MemPoolProvider {
    fn estimate_next_blocktime(&self) -> Task<Result<Duration>>;
    fn collect_deposit_cells(&self) -> Task<Result<Vec<DepositInfo>>>;
    fn query_available_custodians(
        &self,
        withdrawals: Vec<WithdrawalRequest>,
        last_finalized_block_number: u64,
        rollup_context: RollupContext,
    ) -> Task<Result<CollectedCustodianCells>>;
}
