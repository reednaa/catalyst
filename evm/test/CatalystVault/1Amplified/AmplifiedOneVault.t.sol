// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "../../../src/ICatalystV1Vault.sol";
import "../../../src/CatalystVaultAmplified.sol";

import "../Invariant.t.sol";
import "../LocalSwap/LocalSwap.t.sol";
import "../LocalSwap/LocalSwap.minout.t.sol";
import "../LocalSwap/LocalSwap.fees.t.sol";
import "../non-exploits/LocalSwap.SwapWorthlessToken.t.sol";
import "../Set/SetVaultFee.t.sol";
import "../Set/SetGovernanceFee.t.sol";
import "../Setup/Setup.t.sol";
import "../Setup/SetupFinish.t.sol";
import "../CrossChainInterfaceOnly.t.sol";
import "../TokenInterface.t.sol";
import "../Escrow.t.sol";
import "../Withdraw/WithdrawCompare.t.sol";
import "../Withdraw/WithdrawInvariant.t.sol";
import { TestCompareDepositWithWithdraw } from "../Deposit/DepositWithdrawCompare.t.sol";
import { TestWithdrawNothing } from "../Withdraw/WithdrawNothing.t.sol";
import { TestWithdrawUnbalanced } from "../Withdraw/WithdrawUnbalanced.t.sol";
import { TestSecurityLimitAssetSwap } from "../SecurityLimit.ReceiveAsset.t.sol";
import { TestSecurityLimitLiquiditySwap } from "../SecurityLimit.ReceiveLiquidity.t.sol";
import { TestSelfSwap } from "../SelfSwap.t.sol";
import { TestLargeDepositWithdraw } from "../non-exploits/LargeDepositWithdraw.t.sol";
import { TestWithdrawEverything } from "../Withdraw/WithdrawEverything.t.sol";
import {Token} from "../../mocks/token.sol";

contract TestAmplifiedInvariant is TestInvariant, TestLocalswap, TestCrossChainInterfaceOnly, TestLocalswapMinout, TestPoolTokenInterface, TestSetup, TestSetupFinish, TestSetVaultFee, TestSetGovernanceFee, TestLocalswapFees, TestSwapWorthlessTokenLocal, TestEscrow, TestWithdrawInvariant, TestWithdrawComparison, TestCompareDepositWithWithdraw, TestWithdrawNothing, TestWithdrawUnbalanced, TestSelfSwap, TestSecurityLimitAssetSwap, TestSecurityLimitLiquiditySwap, TestLargeDepositWithdraw, TestWithdrawEverything {
    address[] _vaults;

    function setUp() virtual override public {
        super.setUp();

        amplified = true;

        address[] memory assets = getTokens(3);
        uint256[] memory init_balances = new uint256[](3);
        init_balances[0] = 10 * 10**18; init_balances[1] = 100 * 10**18; init_balances[2] = 1000 * 10**18;
        uint256[] memory weights = new uint256[](3);
        weights[0] = 100; weights[1] = 10; weights[2] = 1;

        address vault1 = deployVault(assets, init_balances, weights, 10**18 / 2, 0);

        _vaults.push(vault1);
    }

    function getLargestSwap(address fromVault, address toVault, address fromAsset, address toAsset) view override internal returns(uint256 amount) {
        return getLargestSwap(fromVault, toVault, fromAsset, toAsset, false);
    }

    function getLargestSwap(address fromVault, address toVault, address fromAsset, address toAsset, bool securityLimit) view override internal returns(uint256 amount) {
        uint256 fromWeight = ICatalystV1Vault(fromVault)._weight(fromAsset);
        uint256 toWeight = ICatalystV1Vault(toVault)._weight(toAsset);

        if (securityLimit) {
            amount = Token(toAsset).balanceOf(toVault) * toWeight / fromWeight / 2;
        } else {
            amount = Token(toAsset).balanceOf(toVault) * toWeight / fromWeight;
        }
        uint256 amount2 = Token(fromAsset).balanceOf(address(this));
        if (amount2 < amount) amount = amount2;
    }
    
    function invariant(address[] memory vaults) view internal override returns(uint256 inv) {
        (uint256[] memory balances, uint256[] memory weights) = getBalances(vaults);

        int256 oneMinusAmp = CatalystVaultAmplified(vaults[0])._oneMinusAmp();

        balances = xProduct(balances, weights);

        balances = powerArray(balances, oneMinusAmp);

        inv = getSum(balances);
    }

    // Uses the invariant \sum (i · W)^(1-amp) / \sum (i_0 · W)^(1-amp) = constant for deposits and withdrawals.
    // TODO: Fix
    function strong_invariant(address vault) view internal override returns(uint256 inv) {
        address[] memory vaults = new address[](1);
        vaults[0] = vault;
        (uint256[] memory balances, uint256[] memory weights) = getBalances(vaults);

        // Get the number of tokens.
        uint256 numTokens = balances.length;

        int256 oneMinusAmp = CatalystVaultAmplified(vaults[0])._oneMinusAmp();

        uint256 balance0 = CatalystVaultAmplified(vault).computeBalance0();

        uint256 denum = balance0 * numTokens;

        balances = xProduct(balances, weights);

        balances = powerArray(balances, oneMinusAmp);

        inv = getSum(balances) / denum;
    }

    function getTestConfig() internal override view returns(address[] memory vaults) {
        return vaults = _vaults;
    }
}

