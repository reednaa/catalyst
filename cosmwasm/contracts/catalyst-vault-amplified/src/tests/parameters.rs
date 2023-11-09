use cosmwasm_std::{Uint64, Uint128};

// Define the testing parameters
pub const AMPLIFICATION: Uint64 = Uint64::new(900000000000000000u64);

pub const DECIMALS_6: u128 = 1000000u128;
// pub const DECIMALS_18: u128 = 1000000000000000000u128;

pub const TEST_VAULT_ASSET_COUNT: usize = 3usize;

pub const TEST_VAULT_BALANCES: &'static [Uint128] = &[
    Uint128::new(100u128 * DECIMALS_6),
    Uint128::new(20u128 * DECIMALS_6),
    Uint128::new(3u128 * DECIMALS_6)
];

// The following weights have been chosen such that the product of each weight-balance pair is 6000000e18
pub const TEST_VAULT_WEIGHTS: &'static [Uint128] = &[
    Uint128::new(60000u128),
    Uint128::new(300000u128),
    Uint128::new(2000000u128)
];