use cosmwasm_std::{Uint128, Addr};
use cw20::{Cw20Coin, MinterResponse, BalanceResponse, Cw20QueryMsg, TokenInfoResponse, Cw20ExecuteMsg};
use cw_multi_test::{App, ContractWrapper, AppResponse, Executor};

pub const WAD: Uint128 = Uint128::new(1000000000000000000u128);

#[derive(Clone)]
pub struct TestTokenDefinition {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_mint: Uint128
}

impl TestTokenDefinition {

    fn deploy_token(
        &self,
        app: &mut App,
        cw20_contract: u64,
        minter: Addr
    ) -> Addr {
        app.instantiate_contract::<cw20_base::msg::InstantiateMsg, _>(
            cw20_contract,
            minter.clone(),
            &cw20_base::msg::InstantiateMsg {
                name: self.name.clone(),
                symbol: self.symbol.clone(),
                decimals: self.decimals,
                initial_balances: vec![Cw20Coin {
                    address: minter.to_string(),
                    amount: self.initial_mint
                }],
                mint: Some(MinterResponse {
                    minter: minter.to_string(),
                    cap: None
                }),
                marketing: None
            },
            &[],
            self.symbol.clone(),
            None
        ).unwrap()
    }

}


pub fn mock_test_token_definitions(count: usize) -> Vec<TestTokenDefinition> {
    vec![
        TestTokenDefinition {
            name: "Test Token A".to_string(),
            symbol: "TTA".to_string(),
            decimals: 18,
            initial_mint: Uint128::from(100000000u64) * WAD
        },
        TestTokenDefinition {
            name: "Test Token B".to_string(),
            symbol: "TTB".to_string(),
            decimals: 18,
            initial_mint: Uint128::from(100000000u64) * WAD
        },
        TestTokenDefinition {
            name: "Test Token C".to_string(),
            symbol: "TTC".to_string(),
            decimals: 18,
            initial_mint: Uint128::from(100000000u64) * WAD
        },
        TestTokenDefinition {
            name: "Test Token D".to_string(),
            symbol: "TTD".to_string(),
            decimals: 18,
            initial_mint: Uint128::from(100000000u64) * WAD
        },
        TestTokenDefinition {
            name: "Test Token E".to_string(),
            symbol: "TTE".to_string(),
            decimals: 18,
            initial_mint: Uint128::from(100000000u64) * WAD
        }
    ][0..count].to_vec()
}


pub fn cw20_contract_storage(
    app: &mut App
) -> u64 {

    // Create contract wrapper
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query
    );

    // 'Deploy' the contract
    app.store_code(Box::new(contract))
}


pub fn deploy_test_tokens(
    app: &mut App,
    minter: Addr,
    cw20_contract: Option<u64>,
    token_definitions: Option<Vec<TestTokenDefinition>>
) -> Vec<Addr> {

    let cw20_contract = cw20_contract.unwrap_or(cw20_contract_storage(app));

    token_definitions
        .unwrap_or(mock_test_token_definitions(3))
        .iter()
        .map(|definition| {
            definition.deploy_token(app, cw20_contract, minter.clone())
        })
        .collect()
}


pub fn query_token_balance(
    app: &mut App,
    asset: Addr,
    account: String
) -> Uint128 {
    
    app.wrap().query_wasm_smart::<BalanceResponse>(
        asset,
        &Cw20QueryMsg::Balance { address: account }
    ).unwrap().balance

}


pub fn query_token_info(
    app: &mut App,
    asset: Addr
) -> TokenInfoResponse {
    
    app.wrap().query_wasm_smart::<TokenInfoResponse>(
        asset,
        &Cw20QueryMsg::TokenInfo {}
    ).unwrap()

}


pub fn set_token_allowance(
    app: &mut App,
    amount: Uint128,
    asset: Addr,
    account: Addr,
    spender: String,
) -> AppResponse {
    app.execute_contract::<Cw20ExecuteMsg>(
        account,
        asset,
        &Cw20ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires: None
        },
        &[]
    ).unwrap()
}


pub fn transfer_tokens(
    app: &mut App,
    amount: Uint128,
    asset: Addr,
    account: Addr,
    recipient: String
) -> AppResponse {
    app.execute_contract::<Cw20ExecuteMsg>(
        account,
        asset,
        &Cw20ExecuteMsg::Transfer {
            recipient,
            amount
        },
        &[]
    ).unwrap()
}
