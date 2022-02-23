use dfn_core::api::print;
use std::borrow::Borrow;
use std::str::FromStr;

use crate::named_canister_ids::CANISTER_NAME_LEDGER;
use crate::{get_named_get_canister_id, STATE};
use dfn_core::CanisterId;
use dfn_protobuf::{protobuf, ToProto};
use ledger_canister::protobuf::{ArchiveIndexResponse, TipOfChainRequest};
use ledger_canister::{
    AccountBalanceArgs, BlockHeight, CyclesResponse, EncodedBlock, GetBlocksArgs, GetBlocksRes,
    ICPTs, NotifyCanisterArgs, SendArgs, TipOfChainRes,
};

pub fn get_ledger_id() -> CanisterId {
    CanisterId::from_str(
        get_named_get_canister_id(CANISTER_NAME_LEDGER)
            .to_string()
            .as_str(),
    )
    .unwrap()
}

pub async fn send(request: SendArgs) -> Result<BlockHeight, String> {
    dfn_core::call(get_ledger_id(), "send_pb", protobuf, request.into_proto())
        .await
        .map_err(|e| e.1)
}

pub async fn account_balance(request: AccountBalanceArgs) -> Result<ICPTs, String> {
    dfn_core::call(
        get_ledger_id(),
        "account_balance_pb",
        protobuf,
        request.into_proto(),
    )
    .await
    .map_err(|e| e.1)
}

pub async fn tip_of_chain() -> Result<BlockHeight, String> {
    let response: TipOfChainRes = dfn_core::call(
        get_ledger_id(),
        "tip_of_chain_pb",
        protobuf,
        TipOfChainRequest {},
    )
    .await
    .map_err(|e| e.1)?;

    Ok(response.tip_index)
}

pub async fn get_archive_index() -> Result<ArchiveIndexResponse, String> {
    dfn_core::call(get_ledger_id(), "get_archive_index_pb", protobuf, ())
        .await
        .map_err(|e| e.1)
}

pub async fn get_blocks(
    canister_id: CanisterId,
    from: BlockHeight,
    length: u32,
) -> Result<Vec<EncodedBlock>, String> {
    let response: GetBlocksRes = dfn_core::call(
        canister_id,
        "get_blocks_pb",
        protobuf,
        GetBlocksArgs::new(from, length as usize),
    )
    .await
    .map_err(|e| e.1)?;

    Ok(response.0?)
}
