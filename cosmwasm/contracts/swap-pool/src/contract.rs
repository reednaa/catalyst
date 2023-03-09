use fixed_point_math_lib::fixed_point_math_x64::LN2_X64;
#[cfg(not(feature = "library"))]
use fixed_point_math_lib::{u256::U256, fixed_point_math_x64::ONE_X64};
use cosmwasm_std::{entry_point, Addr, StdError};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, CosmosMsg, to_binary};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info,
};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};
use swap_pool_common::state::{MAX_ASSETS, INITIAL_MINT_AMOUNT, SwapPoolState, STATE};

use crate::calculation_helpers;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:catalyst-swap-pool";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    SwapPoolState::setup(
        &mut deps,
        &env,
        msg.name,
        msg.symbol,
        msg.chain_interface,
        msg.pool_fee,
        msg.governance_fee,
        msg.fee_administrator,
        msg.setup_master
    ).map_err(|err| err.into())

}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitializeSwapCurves {
            assets,
            assets_balances,
            weights,
            amp,
            depositor
        } => execute_initialize_swap_curves(
            deps,
            env,
            info,
            assets,
            assets_balances,
            weights,
            amp,
            depositor
        ),
        ExecuteMsg::FinishSetup {} => execute_finish_setup(deps, info),
        ExecuteMsg::SetFeeAdministrator { administrator } => execute_set_fee_administrator(deps, info, administrator),
        ExecuteMsg::SetPoolFee { fee } => execute_set_pool_fee(deps, info, fee),
        ExecuteMsg::SetGovernanceFee { fee } => execute_set_governance_fee(deps, info, fee),
        ExecuteMsg::SetConnection { channel_id, to_pool, state } => execute_set_connection(deps, info, channel_id, to_pool, state),
        ExecuteMsg::SendAssetAck {
            to_account,
            u,
            amount,
            asset,
            block_number_mod
        } => execute_send_asset_ack(deps, info, to_account, u, amount, asset, block_number_mod),
        ExecuteMsg::SendAssetTimeout {
            to_account,
            u,
            amount,
            asset,
            block_number_mod
        } => execute_send_asset_timeout(deps, env, info, to_account, u, amount, asset, block_number_mod),
        ExecuteMsg::SendLiquidityAck {
            to_account,
            u,
            amount,
            block_number_mod
        } => execute_send_liquidity_ack(deps, info, to_account, u, amount, block_number_mod),
        ExecuteMsg::SendLiquidityTimeout {
            to_account,
            u,
            amount,
            block_number_mod
        } => execute_send_liquidity_timeout(deps, env, info, to_account, u, amount, block_number_mod),
        // ExecuteMsg::Deposit { pool_tokens_amount } => execute_deposit(deps, env, info, pool_tokens_amount),
        // ExecuteMsg::Withdraw { pool_tokens_amount } => execute_withdraw(deps, env, info, pool_tokens_amount),
        // ExecuteMsg::Localswap {
        //     from_asset, //TODO use asset index? - No need to find the index of the asset + consistent with swap_to_units and swap_from_units
        //     to_asset,   //TODO use asset index? - No need to find the index of the asset + consistent with swap_to_units and swap_from_units
        //     amount,
        //     min_out,
        //     approx
        // } => execute_local_swap(deps, env, info, from_asset, to_asset, amount, min_out, approx),
        // ExecuteMsg::SwapToUnits {
        //     chain,
        //     target_pool,
        //     target_user,
        //     from_asset,
        //     to_asset_index,
        //     amount,
        //     min_out,
        //     approx,
        //     fallback_address,
        //     calldata
        // } => execute_swap_to_units(
        //     deps,
        //     env,
        //     info,
        //     chain,
        //     target_pool,
        //     target_user,
        //     from_asset,
        //     to_asset_index,
        //     amount,
        //     min_out,
        //     approx,
        //     fallback_address,
        //     calldata
        // ),

        // CW20 execute msgs - Use cw20-base for the implementation
        ExecuteMsg::Transfer { recipient, amount } => Ok(execute_transfer(deps, env, info, recipient, amount)?),
        ExecuteMsg::Burn { amount } => Err(ContractError::Unauthorized {}),     // Pool token burn handled by withdraw function
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(deps, env, info, contract, amount, msg)?),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => Ok(execute_transfer_from(
            deps, env, info, owner, recipient, amount,
        )?),
        ExecuteMsg::BurnFrom { owner, amount } => Err(ContractError::Unauthorized {}),  // Pool token burn handled by withdraw function
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => Ok(execute_send_from(
            deps, env, info, owner, contract, amount, msg,
        )?),
    }
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ready {} => to_binary(&query_ready(deps)?),
        QueryMsg::OnlyLocal {} => to_binary(&query_only_local(deps)?),
        QueryMsg::GetUnitCapacity {} => to_binary(&query_get_unit_capacity(deps, env)?),

        // CW20 query msgs - Use cw20-base for the implementation
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::Allowance { owner, spender } => to_binary(&query_allowance(deps, owner, spender)?)
    }
}

pub fn execute_initialize_swap_curves(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<String>,
    assets_balances: Vec<Uint128>,  //TODO EVM MISMATCH
    weights: Vec<u64>,
    amp: u64,
    depositor: String
) -> Result<Response, ContractError> {

    let mut state = STATE.load(deps.storage)?;

    // Check the caller is the Factory
    //TODO verify info sender is Factory

    // Make sure this function may only be invoked once (check whether assets have already been saved)
    if state.assets.len() > 0 {
        return Err(ContractError::Unauthorized {});
    }

    // Check that the amplification is correct (set to 1)
    if amp != 10u64.pow(18) {     //TODO maths WAD
        return Err(ContractError::InvalidAmplification {})
    }

    // Check the provided assets and weights count
    if
        assets.len() == 0 || assets.len() > MAX_ASSETS ||
        weights.len() != assets.len()
    {
        return Err(ContractError::GenericError {}); //TODO error
    }

    // Validate the depositor address
    deps.api.addr_validate(&depositor)?;

    // Validate and save assets
    state.assets = assets
        .iter()
        .map(|asset_addr| deps.api.addr_validate(&asset_addr))
        .collect::<StdResult<Vec<Addr>>>()
        .map_err(|_| ContractError::InvalidAssets {})?;

    // Validate asset balances
    if assets_balances.iter().any(|balance| balance.is_zero()) {
        return Err(ContractError::GenericError {}); //TODO error
    }

    // Validate and save weights
    if weights.iter().any(|weight| *weight == 0) {
        return Err(ContractError::GenericError {}); //TODO error
    }
    state.weights = weights.clone();

    // Compute the security limit
    state.max_limit_capacity = (
        LN2_X64 * weights.iter().fold(
            U256::zero(), |acc, next| acc + U256::from(*next)     // Overflow safe, as U256 >> u64    //TODO maths
        )
    ).0;

    // Save state
    STATE.save(deps.storage, &state)?;

    // Mint pool tokens for the depositor
    // Make up a 'MessageInfo' with the sender set to this contract itself => this is to allow the use of the 'execute_mint'
    // function as provided by cw20-base, which will match the 'sender' of 'MessageInfo' with the allowed minter that
    // was set when initializing the cw20 token (this contract itself).
    let execute_mint_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    let minted_amount = INITIAL_MINT_AMOUNT;
    execute_mint(
        deps,
        env.clone(),
        execute_mint_info,
        depositor.clone(),
        minted_amount
    )?;

    // TODO EVM MISMATCH // TODO overhaul: are tokens transferred from the factory? Or will they already be hold by the contract at this point?
    // Build messages to order the transfer of tokens from setup_master to the swap pool
    let sender_addr_str = info.sender.to_string();
    let self_addr_str = env.contract.address.to_string();
    let transfer_msgs: Vec<CosmosMsg> = state.assets.iter().zip(&assets_balances).map(|(asset, balance)| {
        Ok(CosmosMsg::Wasm(
            cosmwasm_std::WasmMsg::Execute {
                contract_addr: asset.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: sender_addr_str.clone(),
                    recipient: self_addr_str.clone(),
                    amount: *balance
                })?,
                funds: vec![]
            }
        ))
    }).collect::<StdResult<Vec<CosmosMsg>>>()?;

    //TODO include attributes of the execute_mint response in this response?
    Ok(
        Response::new()
            .add_messages(transfer_msgs)
            .add_attribute("to_account", depositor)
            .add_attribute("mint", minted_amount)
            .add_attribute("assets", format!("{:?}", assets_balances))
    )
}

pub fn execute_finish_setup(mut deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    SwapPoolState::finish_setup(&mut deps, info).map_err(|err| err.into())
}


// pub fn execute_finish_setup(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo
// ) -> Result<Response, ContractError> {

//     let mut state = STATE.load(deps.storage)?;

//     // Verify the caller
//     let setup_master = state.setup_master.ok_or(ContractError::Unauthorized {})?;
//     if info.sender != setup_master {
//         return Err(ContractError::Unauthorized {});
//     }

//     // Make sure the pool has been properly set-up (some assets have been added, and pool tokens have been minted)  //TODO add any more checks?
//     if query_token_info(deps.as_ref())?.total_supply.is_zero() {
//         return Err(ContractError::Unauthorized {});        
//     }

//     // Set setup_master to None (prevent any further changes)
//     state.setup_master = None;
//     STATE.save(deps.storage, &state)?;

//     Ok(Response::new())
// }


pub fn execute_set_fee_administrator(
    mut deps: DepsMut,
    info: MessageInfo,
    administrator: String
) -> Result<Response, ContractError> {
    SwapPoolState::set_fee_administrator(&mut deps, info, administrator).map_err(|err| err.into())
}


pub fn execute_set_pool_fee(
    mut deps: DepsMut,
    info: MessageInfo,
    fee: u64
) -> Result<Response, ContractError> {
    SwapPoolState::set_pool_fee(&mut deps, info, fee).map_err(|err| err.into())
}


pub fn execute_set_governance_fee(
    mut deps: DepsMut,
    info: MessageInfo,
    fee: u64
) -> Result<Response, ContractError> {
    SwapPoolState::set_governance_fee(&mut deps, info, fee).map_err(|err| err.into())
}


pub fn execute_set_connection(
    mut deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    to_pool: String,
    state: bool
) -> Result<Response, ContractError> {
    SwapPoolState::set_connection(&mut deps, info, channel_id, to_pool, state).map_err(|err| err.into())
}

pub fn execute_send_asset_ack(
    mut deps: DepsMut,
    info: MessageInfo,
    to_account: String,
    u: [u64; 4],
    amount: Uint128,
    asset: String,
    block_number_mod: u32
) -> Result<Response, ContractError> {

    let common_response = SwapPoolState::send_asset_ack(
        &mut deps,
        info,
        to_account,
        U256(u),
        amount,
        asset,
        block_number_mod
    ).map_err(|err| err.into());

    // TODO better approach at implementing this? Currently storage is read and written twice
    // TODO once in send_asset_ack and once to adjust the security limit
    // Adjust security limit
    let mut state = STATE.load(deps.storage)?;

    let used_capacity = U256(state.used_limit_capacity);
    state.used_limit_capacity = (used_capacity.saturating_sub(U256(u))).0;

    STATE.save(deps.storage, &state)?;

    common_response
}

pub fn execute_send_asset_timeout(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_account: String,
    u: [u64; 4],
    amount: Uint128,
    asset: String,
    block_number_mod: u32
) -> Result<Response, ContractError> {
    SwapPoolState::send_asset_timeout(
        &mut deps,
        env,
        info,
        to_account,
        U256(u),
        amount,
        asset,
        block_number_mod
    ).map_err(|err| err.into())
}

pub fn execute_send_liquidity_ack(
    mut deps: DepsMut,
    info: MessageInfo,
    to_account: String,
    u: [u64; 4],
    amount: Uint128,
    block_number_mod: u32
) -> Result<Response, ContractError> {

    let common_response = SwapPoolState::send_liquidity_ack(
        &mut deps,
        info,
        to_account,
        U256(u),
        amount,
        block_number_mod
    ).map_err(|err| err.into());

    // TODO better approach at implementing this? Currently storage is read and written twice
    // TODO once in send_asset_ack and once to adjust the security limit
    // Adjust security limit
    let mut state = STATE.load(deps.storage)?;

    let used_capacity = U256(state.used_limit_capacity);
    state.used_limit_capacity = (used_capacity.saturating_sub(U256(u))).0;

    STATE.save(deps.storage, &state)?;

    common_response
}

pub fn execute_send_liquidity_timeout(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_account: String,
    u: [u64; 4],
    amount: Uint128,
    block_number_mod: u32
) -> Result<Response, ContractError> {
    SwapPoolState::send_liquidity_timeout(
        &mut deps,
        env,
        info,
        to_account,
        U256(u),
        amount,
        block_number_mod
    ).map_err(|err| err.into())
}



// pub fn execute_deposit(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     pool_tokens_amount: Uint128
// ) -> Result<Response, ContractError> {

//     let mut state = STATE.load(deps.storage)?;

//     let pool_token_supply = get_pool_token_supply(deps.as_ref())?;

//     // Update the liqudity security limit. Since the limit is based on the current totalSupply, changing the totalSupply
//     // upwards by depositing changes the limit.
//     let current_timestamp: u64 = env.block.time.seconds();
//     state.update_liquidity_units_inflow(
//         Uint128::zero(),    // No pool tokens are going into the pool via a bridge
//         pool_token_supply,
//         current_timestamp
//     )?;
    
//     // Given the desired 'pool_tokens_amount', compute the corresponding deposit amounts for each of the assets of the pool
//     let balance_msg = Cw20QueryMsg::Balance { address: env.contract.address.to_string() };

//     let deposited_amounts: Vec<Uint128> = state.assets.iter().enumerate().map(|(asset_index, asset)| {

//         // Get the pool's asset balance
//         let swap_pool_asset_balance = deps.querier.query_wasm_smart(
//             asset,
//             &balance_msg
//         )?;

//         // Determine the share of the common 'pool_tokens_amount' that corresponds to this asset
//         let asset_balance0 = state.assets_balance0s[asset_index];
//         let pool_tokens_for_asset = pool_tokens_amount
//             .checked_mul(asset_balance0).map_err(|_| ContractError::ArithmeticError {})?
//             .checked_div(pool_token_supply).map_err(|_| ContractError::ArithmeticError {})?;

//         // Determine the asset deposit amout
//         let asset_deposit_amount = calculation_helpers::calc_asset_amount_for_pool_tokens(
//             pool_tokens_for_asset,
//             swap_pool_asset_balance,     // Escrowed tokens are NOT subtracted from the total balance => deposits should return less
//             asset_balance0
//         )?;
        
//         // Update the asset balance0
//         state.assets_balance0s[asset_index] = 
//             asset_balance0.checked_add(pool_tokens_for_asset).map_err(|_| ContractError::ArithmeticError {})?;
        
//         Ok(asset_deposit_amount)

//     }).collect::<Result<Vec<Uint128>, ContractError>>()?;

//     // Save state
//     STATE.save(deps.storage, &state)?;


//     // Mint pool tokens for the depositor
//     let depositor_addr_str = info.sender.to_string();
//     let self_addr_str = env.contract.address.to_string();

//     // Make up a 'MessageInfo' with the sender set to this contract itself => this is to allow the use of the 'execute_mint'
//     // function as provided by cw20-base, which will match the 'sender' of 'MessageInfo' with the allowed minter that
//     // was set when initializing the cw20 token (this contract itself).
//     let execute_mint_info = MessageInfo {
//         sender: env.contract.address.clone(),
//         funds: vec![],
//     };
//     execute_mint(deps, env.clone(), execute_mint_info, self_addr_str.clone(), pool_tokens_amount)?;


//     // TODO move transfer functionality somewhere else? Implement on 'SwapPoolState'?
//     // Build messages to order the transfer of tokens from the depositor to the swap pool
//     let transfer_msgs: Vec<CosmosMsg> = state.assets.iter().zip(&deposited_amounts).map(|(asset, balance)| {
//         Ok(CosmosMsg::Wasm(
//             cosmwasm_std::WasmMsg::Execute {
//                 contract_addr: asset.to_string(),
//                 msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
//                     owner: depositor_addr_str.clone(),
//                     recipient: self_addr_str.clone(),
//                     amount: *balance
//                 })?,
//                 funds: vec![]
//             }
//         ))
//     }).collect::<StdResult<Vec<CosmosMsg>>>()?;

//     Ok(
//         Response::new()
//             .add_messages(transfer_msgs)
//             .add_attribute("deposited_amounts", format!("{:?}", deposited_amounts))
//             .add_attribute("minted_pool_tokens", pool_tokens_amount)
//     )
// }


// pub fn execute_withdraw(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     pool_tokens_amount: Uint128
// ) -> Result<Response, ContractError> {

//     let mut state = STATE.load(deps.storage)?;

//     let pool_token_supply = get_pool_token_supply(deps.as_ref())?;

//     // Update the liqudity security limit. Since the limit is based on the current totalSupply, changing the totalSupply
//     // downwards by withdrawing changes the limit.
//     let current_timestamp: u64 = env.block.time.seconds();
//     state.update_liquidity_units_inflow(
//         Uint128::zero(),    // No pool tokens are going into the pool via a bridge
//         pool_token_supply,
//         current_timestamp
//     )?;
    
//     // Given the desired 'pool_tokens_amount', compute the corresponding withdraw amounts for each of the assets of the pool
//     let balance_msg = Cw20QueryMsg::Balance { address: env.contract.address.to_string() };

//     let withdrawal_amounts: Vec<Uint128> = state.assets.iter().enumerate().map(|(asset_index, asset)| {

//         // Get the pool's asset balance
//         let swap_pool_asset_balance: Uint128 = deps.querier.query_wasm_smart(
//             asset,
//             &balance_msg
//         )?;

//         // Determine the share of the common 'pool_tokens_amount' that corresponds to this asset
//         let asset_balance0 = state.assets_balance0s[asset_index];
//         let pool_tokens_for_asset = pool_tokens_amount
//             .checked_mul(asset_balance0).map_err(|_| ContractError::ArithmeticError {})?
//             .checked_div(pool_token_supply).map_err(|_| ContractError::ArithmeticError {})?;

//         // Determine the asset withdrawal amount
//         let asset_withdrawal_amount = calculation_helpers::calc_asset_amount_for_pool_tokens(
//             pool_tokens_for_asset,
//             swap_pool_asset_balance
//                 .checked_sub(state.escrowed_assets[asset_index]).map_err(|_| ContractError::ArithmeticError {})?,         // Escrowed tokens ARE subtracted from the total balance => withdrawals should return less
//             asset_balance0
//         )?;
        
//         // Update the asset balance0
//         state.assets_balance0s[asset_index] = 
//             asset_balance0.checked_sub(pool_tokens_for_asset).map_err(|_| ContractError::ArithmeticError {})?;
        
//         Ok(asset_withdrawal_amount)

//     }).collect::<Result<Vec<Uint128>, ContractError>>()?;

//     // Save state
//     STATE.save(deps.storage, &state)?;



//     // Burn pool tokens from the withdrawer
//     let withdrawer_addr_str = info.sender.to_string();
//     let self_addr_str = env.contract.address.to_string();

//     execute_burn_from(deps, env.clone(), info, withdrawer_addr_str.clone(), pool_tokens_amount)?;

    
//     // TODO move transfer functionality somewhere else? Implement on 'SwapPoolState'?
//     // Build messages to order the transfer of tokens from the depositor to the swap pool
//     let transfer_msgs: Vec<CosmosMsg> = state.assets.iter().zip(&withdrawal_amounts).map(|(asset, balance)| {
//         Ok(CosmosMsg::Wasm(
//             cosmwasm_std::WasmMsg::Execute {
//                 contract_addr: asset.to_string(),
//                 msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
//                     owner: self_addr_str.clone(),
//                     recipient: withdrawer_addr_str.clone(),
//                     amount: *balance
//                 })?,
//                 funds: vec![]
//             }
//         ))
//     }).collect::<StdResult<Vec<CosmosMsg>>>()?;

    
//     Ok(
//         Response::new()
//             .add_messages(transfer_msgs)
//             .add_attribute("withdrawn_amounts", format!("{:?}", withdrawal_amounts))
//             .add_attribute("burnt_pool_tokens", pool_tokens_amount)
//     )

// }


// pub fn execute_local_swap(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     from_asset: String,
//     to_asset: String,
//     amount: Uint128,
//     min_out: Uint128,
//     approx: bool
// ) -> Result<Response, ContractError> {

//     let state = STATE.load(deps.storage)?;

//     // No need to verify whether from_asset or to_asset are valid addresses, as they must match one of the
//     // addresses saved to the 'assets' vec of the swap pool state (otherwise get_asset_index will fail).
//     let from_asset_index = state.get_asset_index(&from_asset)?;
//     let to_asset_index = state.get_asset_index(&to_asset)?;

//     // Query the asset balances
//     let balance_msg = Cw20QueryMsg::Balance { address: env.contract.address.to_string() };
//     let from_asset_balance: Uint128 = deps.querier.query_wasm_smart(&from_asset, &balance_msg)?;
//     let to_asset_balance: Uint128 = deps.querier.query_wasm_smart(&to_asset, &balance_msg)?;
    
//     // Calculate swap output
//     let out = Uint128::from(calculation_helpers::full_swap(
//         U256::from(amount.u128()),
//         U256::from(from_asset_balance.u128()),
//         U256::from(state.weights[from_asset_index]),
//         U256::from(
//             to_asset_balance
//                 .checked_sub(state.escrowed_assets[to_asset_index])
//                 .map_err(|_| ContractError::ArithmeticError {})?
//                 .u128()
//             ),
//         U256::from(state.weights[to_asset_index]),
//         approx
//     )?.as_u128());      // U256 to u64 will panic if overflow

//     if out < min_out { return Err(ContractError::SwapMinYieldNotFulfilled {}) }

    
//     // Build message to transfer input assets to the pool
//     let swapper_addr_str = info.sender.to_string();
//     let self_addr_str    = env.contract.address.to_string();

//     let transfer_from_asset_msg = CosmosMsg::Wasm(
//         cosmwasm_std::WasmMsg::Execute {
//             contract_addr: from_asset.clone(),
//             msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
//                 owner: swapper_addr_str.clone(),
//                 recipient: self_addr_str.clone(),
//                 amount
//             })?,
//             funds: vec![]
//         }
//     );

//     // Build message to transfer output assets to the swapper
//     let transfer_to_asset_msg = CosmosMsg::Wasm(
//         cosmwasm_std::WasmMsg::Execute {
//             contract_addr: from_asset.clone(),
//             msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
//                 owner: self_addr_str,
//                 recipient: swapper_addr_str.clone(),
//                 amount: out
//             })?,
//             funds: vec![]
//         }
//     );

    
//     Ok(
//         Response::new()
//             .add_message(transfer_from_asset_msg)
//             .add_message(transfer_to_asset_msg)
//             .add_attribute("from_asset", from_asset)
//             .add_attribute("to_asset", to_asset)
//             .add_attribute("amount", amount)
//             .add_attribute("yield", out)
//             .add_attribute("fees", "0")     //TODO
//     )

// }


// pub fn execute_swap_to_units(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
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
// ) -> Result<Response, ContractError> {

//     let mut state = STATE.load(deps.storage)?;

//     // No need to verify whether from_asset is a valid address, as it must match one of the
//     // addresses saved to the 'assets' vec of the swap pool state (otherwise get_asset_index will fail).
//     let from_asset_index = state.get_asset_index(&from_asset)?;

//     // Query the from_asset balance
//     let balance_msg = Cw20QueryMsg::Balance { address: env.contract.address.to_string() };
//     let from_asset_balance: Uint128 = deps.querier.query_wasm_smart(&from_asset, &balance_msg)?;

//     let units_x64 = calculation_helpers::out_swap_x64(
//         U256::from(amount.u128()),  //TODO subtract pool fee
//         U256::from(from_asset_balance.u128()),
//         U256::from(state.weights[from_asset_index]),
//         (approx & 1) > 0
//     )?;

//     // The hash for the escrow is built only with the data that matters for the escrow (+ the implementation is specific for each implementation)
//     //   - source asset         (32 bytes?)
//     //   - source asset amount  (16 bytes)
//     //   - units                (32 bytes)
//     // To randomize the hash, the following are also included
//     //   - target user          (32 bytes)
//     //   - block number         (8 bytes)
//     //   Note that the fallback user is not included on the hash as it is not sent on the IBC packet
//     //   and hence cannot be recovered for the timeout/ack execution (it is saved to variable)

//     let mut hash_data: Vec<u8> = Vec::with_capacity(32 * 3 + 16 + 8);    // Initialize vec with the specified capacity (avoid reallocations)
//     hash_data.extend_from_slice(from_asset.as_bytes());  // TODO make sure it's 32 bytes
//     hash_data.extend_from_slice(&amount.to_be_bytes());

//     let mut units_x64_bytes = [0u8; 32];
//     units_x64.to_big_endian(units_x64_bytes.as_mut_slice());
//     hash_data.extend_from_slice(&units_x64_bytes);                       //TODO better way to do this?

//     hash_data.extend_from_slice(target_user.as_bytes());
//     hash_data.extend_from_slice(&env.block.height.to_be_bytes());

//     let escrow_hash = calc_keccak256(hash_data);

//     // Verify and save the fallback_address to the escrow
//     if ESCROWS.has(deps.storage, escrow_hash.as_str()) {
//         return Err(ContractError::NonEmptyEscrow {});
//     }

//     let fallback_address = deps.api.addr_validate(fallback_address.as_str())?;
//     ESCROWS.save(deps.storage, &escrow_hash.as_str(), &Escrow{ fallback_address })?;

//     // Escrow the assets
//     state.escrowed_assets[from_asset_index] =
//         state.escrowed_assets[from_asset_index].checked_add(amount).map_err(|_| ContractError::ArithmeticError {})?;    //TODO subtract pool fee


//     // Save swap pool state
//     STATE.save(deps.storage, &state)?;


//     // Build message to transfer assets from the user to the pool
//     let transfer_assets_msg = CosmosMsg::Wasm(
//         cosmwasm_std::WasmMsg::Execute {
//             contract_addr: from_asset.clone(),
//             msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
//                 owner: info.sender.to_string(),
//                 recipient: env.contract.address.to_string(),
//                 amount
//             })?,
//             funds: vec![]
//         }
//     );

//     // TODO Build message to transfer governance fee to governance contract
//     // let transfer_gov_fee_msg = 

//     // TODO Build message to invoke the IBC interface    
//     // let invoke_chain_interface_msg = 

//     Ok(
//         Response::new()
//             .add_message(transfer_assets_msg)
//             // .add_message(transfer_gov_fee_msg)
//             // .add_message(invoke_chain_interface_msg)
//             .add_attribute("target_pool", target_pool)
//             .add_attribute("target_user", target_user)
//             .add_attribute("from_asset", from_asset)
//             .add_attribute("to_asset_index", to_asset_index.to_string())
//             .add_attribute("amount", amount.to_string())
//             .add_attribute("units", format!("{:?}", units_x64.0))   //TODO currently returning an array (made into a string) => return a number
//             .add_attribute("min_out", format!("{:?}", min_out))     //TODO currently returning an array (made into a string) => return a number
//             .add_attribute("escrow_hash", escrow_hash)
//     )

// }


pub fn query_ready(deps: Deps) -> StdResult<bool> {
    SwapPoolState::ready(deps)
}


pub fn query_only_local(deps: Deps) -> StdResult<bool> {
    SwapPoolState::only_local(deps)
}

pub fn query_get_unit_capacity(deps: Deps, env: Env) -> StdResult<[u64; 4]> { //TODO maths
    SwapPoolState::get_unit_capacity(deps, env)
        .map(|capacity| capacity.0)
        .map_err(|_| StdError::GenericErr { msg: "".to_owned() })   //TODO error
}


// //TODO move this fn somewhere else?
// fn get_pool_token_supply(deps: Deps) -> StdResult<Uint128> {
//     let info = TOKEN_INFO.load(deps.storage)?;
//     Ok(info.total_supply)
// }

#[cfg(test)]
mod tests {
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use cosmwasm_std::{Addr, Empty, Uint128, Attribute};
    use cw20::{Cw20Coin, Cw20ExecuteMsg, MinterResponse, Cw20QueryMsg, BalanceResponse};
    use swap_pool_common::{msg::{InstantiateMsg, ExecuteMsg}, state::INITIAL_MINT_AMOUNT};

    pub const INSTANTIATOR_ADDR: &str = "inst_addr";
    pub const OTHER_ADDR: &str = "other_addr";

    pub fn contract_swap_pool() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query
        );
        Box::new(contract)
    }

    //TODO add instantiate tests

    #[test]
    fn test_setup_and_instantiate() {
        //TODO should this be considered an integration test? => Move somewhere else

        let mut router = App::default();

        let setup_master = Addr::unchecked(INSTANTIATOR_ADDR);


        // Create test token
        let cw20_id = router.store_code(contract_cw20());

        let msg = cw20_base::msg::InstantiateMsg {
            name: "Test Token A".to_string(),
            symbol: "TTA".to_string(),
            decimals: 2,
            initial_balances: vec![Cw20Coin {
                address: setup_master.to_string(),
                amount: Uint128::new(5000)
            }],
            mint: Some(MinterResponse {
                minter: setup_master.to_string(),
                cap: None
            }),
            marketing: None
        };

        let test_token_1_addr = router
            .instantiate_contract(cw20_id, setup_master.clone(), &msg, &[], "TTA", None)
            .unwrap();


        // Create swap pool - upload and instantiate
        let sp_id = router.store_code(contract_swap_pool());

        let sp_addr = router
            .instantiate_contract(
                sp_id,
                setup_master.clone(),
                &InstantiateMsg {
                    name: "Pool1".to_owned(),
                    symbol: "P1".to_owned(),
                    chain_interface: None,
                    pool_fee: 0u64,
                    governance_fee: 0u64,
                    fee_administrator: setup_master.to_string(),
                    setup_master: setup_master.to_string(),
                },
                &[],
                "sp",
                None
            ).unwrap();



        // Set allowance for the swap pool
        let deposit_amount = Uint128::from(1000_u64);
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: sp_addr.to_string(),
            amount: deposit_amount,
            expires: None
        };

        router.execute_contract(
            setup_master.clone(),
            test_token_1_addr.clone(),
            &allowance_msg,
            &[]
        ).unwrap();


        // Initialize sp balances
        let initialize_balances_msg = ExecuteMsg::InitializeSwapCurves {
            assets: vec![test_token_1_addr.to_string()],
            assets_balances: vec![Uint128::from(1000_u64)],
            weights: vec![1u64],
            amp: 1000000000000000000u64,
            depositor: setup_master.to_string()
        };

        let response = router.execute_contract(
            setup_master.clone(),
            sp_addr.clone(),
            &initialize_balances_msg,
            &[]
        ).unwrap();

        // Verify attributes
        let initialize_event = response.events[1].clone();
        assert_eq!(
            initialize_event.attributes[1],
            Attribute { key: "to_account".to_string(), value: setup_master.to_string()}
        );
        assert_eq!(
            initialize_event.attributes[2],
            Attribute { key: "mint".to_string(), value: INITIAL_MINT_AMOUNT.to_string()}
        );
        assert_eq!(
            initialize_event.attributes[3],
            Attribute { key: "assets".to_string(), value: format!("{:?}", vec![Uint128::from(1000_u64)])}
        );


        // Verify token balances
        // Swap pool balance of test token 1
        let balance_msg = Cw20QueryMsg::Balance { address: sp_addr.to_string() };
        let balance_response: BalanceResponse = router
            .wrap()
            .query_wasm_smart(test_token_1_addr.clone(), &balance_msg)
            .unwrap();
        assert_eq!(balance_response.balance, Uint128::from(1000_u64));

        // User balance of test token 1
        let balance_msg = Cw20QueryMsg::Balance { address: setup_master.to_string() };
        let balance_response: BalanceResponse = router
            .wrap()
            .query_wasm_smart(test_token_1_addr.clone(), &balance_msg)
            .unwrap();

        assert_eq!(balance_response.balance, Uint128::from(4000_u64));


        // Verify pool token balance
        let balance_msg = Cw20QueryMsg::Balance { address: setup_master.to_string() };
        let balance_response: BalanceResponse = router
            .wrap()
            .query_wasm_smart(sp_addr.clone(), &balance_msg)
            .unwrap();

        assert_eq!(balance_response.balance, INITIAL_MINT_AMOUNT);

    }

    // TODO make sure 'InitializeSwapCurves' can only be called once

}