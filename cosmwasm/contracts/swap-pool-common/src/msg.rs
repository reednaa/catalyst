use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Binary, Uint64, Uint128};
use cw20::{Expiration, AllowanceResponse, BalanceResponse, TokenInfoResponse};


#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,       // Name for the pool token
    pub symbol: String,     // Symbol for the pool token
    pub chain_interface: Option<String>,
    pub pool_fee: u64,
    pub governance_fee: u64,
    pub fee_administrator: String,
    pub setup_master: String,
}


#[cw_serde]
pub enum ExecuteMsg {

    InitializeSwapCurves {
        assets: Vec<String>,
        assets_balances: Vec<Uint128>,
        weights: Vec<u64>,
        amp: u64,
        depositor: String
    },

    FinishSetup {},

    SetPoolFee { fee: u64 },

    SetGovernanceFee { fee: u64 },

    SetFeeAdministrator { administrator: String },

    SetConnection {
        channel_id: String,
        to_pool: String,
        state: bool
    },

    SendAssetAck {
        to_account: String,
        u: [u64; 4],
        amount: Uint128,
        asset: String,
        block_number_mod: u32
    },

    SendAssetTimeout {
        to_account: String,
        u: [u64; 4],
        amount: Uint128,
        asset: String,
        block_number_mod: u32
    },

    SendLiquidityAck {
        to_account: String,
        u: [u64; 4],
        amount: Uint128,
        block_number_mod: u32
    },

    SendLiquidityTimeout {
        to_account: String,
        u: [u64; 4],
        amount: Uint128,
        block_number_mod: u32
    },

    // Setup {
    //     assets: Vec<String>,
    //     weights: Vec<u64>,          // TODO type? (originally u256)
    //     amp: [u64; 4],
    //     governance_fee: [u64; 4],
    //     name: String,
    //     symbol: String,
    //     chain_interface: String,
    //     setup_master: String
    // },

    // Deposit { pool_tokens_amount: Uint128 },

    // Withdraw { pool_tokens_amount: Uint128 },

    // Localswap {
    //     from_asset: String,
    //     to_asset: String,
    //     amount: Uint128,
    //     min_out: Uint128,
    //     approx: bool
    // },

    // SwapToUnits {
    //     chain: u32,
    //     target_pool: String,
    //     target_user: String,
    //     from_asset: String,
    //     to_asset_index: u8,
    //     amount: Uint128,
    //     min_out: [u64; 4],
    //     approx: u8,
    //     fallback_address: String,
    //     calldata: Vec<u8>
    // },

    // SwapFromUnits {
    //     to_asset_index: u8,
    //     who: String,
    //     units: [u64; 4],
    //     min_out: [u64; 4],
    //     approx: bool,
    //     message_hash: String,
    //     data_target: String,
    //     // bytes calldata data // TODO vec<>?
    // },

    // OutLiquidity {
    //     chain: [u64; 4],
    //     target_pool: String,
    //     who: String,
    //     base_amount: [u64; 4],
    //     min_out: [u64; 4],
    //     approx: u8,
    //     fallback_user: String
    // },

    // InLiquidity {
    //     who: String,
    //     units: [u64; 4],
    //     min_out: [u64; 4],
    //     approx: bool,
    //     message_hash: String
    // }


    // CW20 Implementation
    Transfer { recipient: String, amount: Uint128 },
    Burn { amount: Uint128 },
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    BurnFrom { owner: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

    // #[returns(ChainInterfaceResponse)]
    // ChainInterface {},

    // // TokenIndexing(tokenIndex: [u64; 4]),

    // #[returns(Balance0Response)]
    // Balance0 {
    //     token: String
    // },

    // #[returns(WeightResponse)]
    // Weight {
    //     token: String
    // },

    // #[returns(WeightResponse)]
    // TargetWeight{
    //     token: String
    // },

    // #[returns(AdjustmentTargetResponse)]
    // AdjustmentTarget {},

    // #[returns(LastModificationTimeResponse)]
    // LastModificationTime {},

    // #[returns(TargetMaxUnitInflowResponse)]
    // TargetMaxUnitInflow {},

    // #[returns(PoolFeeX64Response)]
    // PoolFeeX64 {},

    // #[returns(GovernanceFeeResponse)]
    // GovernanceFee {},

    // #[returns(FeeAdministratorResponse)]
    // FeeAdministrator {},

    // #[returns(SetupMasterResponse)]
    // SetupMaster {},

    // #[returns(MaxUnitInflowResponse)]
    // MaxUnitInflow {},

    // #[returns(EscrowedTokensResponse)]
    // EscrowedTokens { token: String },

    // #[returns(EscrowedPoolTokensResponse)]
    // EscrowedPoolTokens {},

    // // #[returns(FactoryOwnerResponse)]
    // // FactoryOwner {},

    #[returns(ReadyResponse)]
    Ready {},
    #[returns(OnlyLocalResponse)]
    OnlyLocal {},
    #[returns(GetUnitCapacityResponse)]
    GetUnitCapacity {},


    // CW20 Implementation
    #[returns(BalanceResponse)]
    Balance { address: String },
    #[returns(TokenInfoResponse)]
    TokenInfo {},
    #[returns(AllowanceResponse)]
    Allowance { owner: String, spender: String },

}


#[cw_serde]
pub struct UnitCapacityResponse {
    pub amount: [u64; 4],
}

#[cw_serde]
pub struct LiquidityUnitCapacityResponse {
    pub amount: [u64; 4],
}

#[cw_serde]
pub struct ChainInterfaceResponse {
    pub contract: String,
}

#[cw_serde]
pub struct Balance0Response {
    pub balance: [u64; 4],
}

#[cw_serde]
pub struct WeightResponse {
    pub weight: Uint64,     //TODO TYPE
}

#[cw_serde]
pub struct TargetWeightResponse {
    pub weight: Uint64,     //TODO TYPE
}

#[cw_serde]
pub struct AdjustmentTargetResponse {
    // TODO
}

#[cw_serde]
pub struct LastModificationTimeResponse {
    // TODO
}

#[cw_serde]
pub struct TargetMaxUnitInflowResponse {
    pub amount: [u64; 4]
}

#[cw_serde]
pub struct PoolFeeX64Response {
    pub fee: [u64; 4]    //TODO use u64?
}

#[cw_serde]
pub struct GovernanceFeeResponse {
    pub fee: [u64; 4]    //TODO use u64?
}

#[cw_serde]
pub struct FeeAdministratorResponse {
    pub admin: String
}

#[cw_serde]
pub struct SetupMasterResponse {
    pub setup_master: String
}

#[cw_serde]
pub struct MaxUnitInflowResponse {
    pub amount: [u64; 4]
}

#[cw_serde]
pub struct EscrowedTokensResponse {
    pub amount: Uint128
}

#[cw_serde]
pub struct EscrowedPoolTokensResponse {
    pub amount: Uint128
}

// #[cw_serde]
// pub struct FactoryOwnerResponse {

// }

#[cw_serde]
pub struct ReadyResponse {
    pub ready: Binary
}

#[cw_serde]
pub struct OnlyLocalResponse {
    pub only_local: Binary
}

#[cw_serde]
pub struct GetUnitCapacityResponse {
    pub capacity: [u64; 4]
}


