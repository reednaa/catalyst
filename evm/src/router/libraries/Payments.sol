// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.17;

import {Constants} from '../libraries/Constants.sol';
import {RouterImmutables} from '../base/RouterImmutables.sol';
import {SafeTransferLib} from 'solady/utils/SafeTransferLib.sol';
import {ERC20} from 'solady/tokens/ERC20.sol';

/// @title Payments contract
/// @notice Performs various operations around the payment of ETH and tokens
abstract contract Payments is RouterImmutables {
    using SafeTransferLib for address;

    error InsufficientToken();  // 675cae38
    error InsufficientETH();  // 6a12f104
    error InvalidBips();  // deaa01e6

    uint256 internal constant FEE_BIPS_BASE = 10_000;

    /// @notice Pays an amount of ETH or ERC20 to a recipient
    /// @param token The token to pay (can be ETH using Constants.ETH)
    /// @param recipient The address that will receive the payment
    /// @param value The amount to pay
    function pay(address token, address recipient, uint256 value) internal {
        if (token == Constants.ETH) {
            recipient.safeTransferETH(value);
        } else {
            if (value == Constants.CONTRACT_BALANCE) {
                value = ERC20(token).balanceOf(address(this));
            }

            SafeTransferLib.safeTransfer(token, recipient, value);
        }
    }


    /// @notice Pays a proportion of the contract's ETH or ERC20 to a recipient
    /// @param token The token to pay (can be ETH using Constants.ETH)
    /// @param recipient The address that will receive payment
    /// @param bips Portion in bips of whole balance of the contract
    function payPortion(address token, address recipient, uint256 bips) internal {
        if (bips == 0 || bips > 10_000) revert InvalidBips();
        if (token == Constants.ETH) {
            uint256 balance = address(this).balance;
            uint256 amount = (balance * bips) / FEE_BIPS_BASE;
            recipient.safeTransferETH(amount);
        } else {
            uint256 balance = ERC20(token).balanceOf(address(this));
            uint256 amount = (balance * bips) / FEE_BIPS_BASE;
            // pay with tokens already in the contract (for the exact input multihop case)
            SafeTransferLib.safeTransfer(token, recipient, amount);
        }
    }

    /// @notice Sweeps all of the contract's ERC20 or ETH to an address
    /// @param token The token to sweep (can be ETH using Constants.ETH)
    /// @param recipient The address that will receive payment
    /// @param amountMinimum The minimum desired amount
    function sweep(address token, address recipient, uint256 amountMinimum) internal {
        uint256 balance;
        if (token == Constants.ETH) {
            balance = address(this).balance;
            if (balance < amountMinimum) revert InsufficientETH();
            if (balance > 0) recipient.safeTransferETH(balance);
        } else {
            balance = ERC20(token).balanceOf(address(this));
            if (balance < amountMinimum) revert InsufficientToken();
            if (balance > 0) SafeTransferLib.safeTransfer(token, recipient, balance);
        }
    }

    /// @notice Wraps an amount of ETH into WETH
    /// @param recipient The recipient of the WETH
    /// @param amount The amount to wrap (can be CONTRACT_BALANCE)
    function wrapETH(address recipient, uint256 amount) internal {
        if (amount == Constants.CONTRACT_BALANCE) {
            amount = address(this).balance;
        } else if (amount > address(this).balance) {
            revert InsufficientETH();
        }
        if (amount > 0) {
            WETH9.deposit{value: amount}();
            if (recipient != address(this)) {
                WETH9.transfer(recipient, amount);
            }
        }
    }

    /// @notice Unwraps all of the contract's WETH into ETH
    /// @param recipient The recipient of the ETH
    /// @param amountMinimum The minimum amount of ETH desired
    function unwrapWETH9(address recipient, uint256 amountMinimum) internal {
        uint256 value = WETH9.balanceOf(address(this));
        if (value < amountMinimum) {
            revert InsufficientETH();
        }
        if (value > 0) {
            WETH9.withdraw(value);
            if (recipient != address(this)) {
                recipient.safeTransferETH(value);
            }
        }
    }

    function transferFrom(address token, address from, address to, uint160 amount) internal {
        SafeTransferLib.safeTransferFrom(token, from, to, amount);
    }
}