// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import { TestCommon } from "../TestCommon.t.sol";

import { ICatalystV1Structs } from "../../src/interfaces/ICatalystV1VaultState.sol";
import { ICatalystV1Vault } from "../../src/ICatalystV1Vault.sol";
import { CatalystChainInterface } from "../../src/CatalystChainInterface.sol";
import { Token } from "../mocks/token.sol";

import { ICatalystReceiver } from "../../src/interfaces/IOnCatalyst.sol";

contract TestSendAssetUnderwritePurpose is TestCommon {
    address vault1;
    address vault2;

    bytes32 FEE_RECIPITANT = bytes32(uint256(uint160(0xfee0eec191fa4f)));

    event SwapFailed(bytes1 error);

    event SendAssetSuccess(
        bytes32 channelId,
        bytes toAccount,
        uint256 units,
        uint256 escrowAmount,
        address escrowToken,
        uint32 blockNumberMod
    );

    event SendAssetFailure(
        bytes32 channelId,
        bytes toAccount,
        uint256 units,
        uint256 escrowAmount,
        address escrowToken,
        uint32 blockNumberMod
    );
    event Transfer(address indexed from, address indexed to, uint256 amount);
    
    function setUp() virtual override public {
        super.setUp();
        vault1 = simpleVault(1);
        vault2 = simpleVault(1);

        // Setup
        setConnection(vault1, vault2, DESTINATION_IDENTIFIER, DESTINATION_IDENTIFIER);
    }

    function test_send_asset_underwrite_purpose(address refundTo, address toAccount) external {
        vm.assume(refundTo != address(0));
        vm.assume(refundTo != address(0));
        vm.assume(refundTo != vault1);
        vm.assume(refundTo != vault2);
        vm.assume(refundTo != address(CCI));
        vm.assume(refundTo != address(this));
        vm.assume(toAccount != refundTo);  // makes it really hard to debug
        vm.assume(toAccount != address(0));
        vm.assume(toAccount != vault1);
        vm.assume(toAccount != vault2);
        vm.assume(toAccount != address(CCI));
        vm.assume(toAccount != address(this));
        // execute the swap.
        address token1 = ICatalystV1Vault(vault1)._tokenIndexing(0);

        Token(token1).approve(vault1, 2**256-1);

        ICatalystV1Structs.RouteDescription memory routeDescription = ICatalystV1Structs.RouteDescription({
            chainIdentifier: DESTINATION_IDENTIFIER,
            toVault: convertEVMTo65(vault2),
            toAccount: convertEVMTo65(toAccount),
            incentive: _INCENTIVE
        });

        address token2 = ICatalystV1Vault(vault2)._tokenIndexing(0);


        
        Token(token2).transfer(refundTo, 103489651034896500);
        vm.prank(refundTo);
        Token(token2).approve(address(CCI), 2**256-1);
        vm.prank(refundTo);
        bytes32 underwriteIdentifier = CCI.underwrite(
            vault2,  // -- Swap information
            token2,
            99995000333308,
            0,
            toAccount,
            0,
            hex"0000"
        );


        vm.recordLogs();
        uint256 units = ICatalystV1Vault(vault1).sendAssetFixedUnit{value: _getTotalIncentive(_INCENTIVE)}(
            routeDescription,
            token1,
            0, 
            uint256(1e17), 
            0,
            99995000333308,
            toAccount,
            0,
            hex""
        );
        Vm.Log[] memory entries = vm.getRecordedLogs();

        (, , bytes memory messageWithContext) = abi.decode(entries[1].data, (bytes32, bytes, bytes));

        assertEq(units, 99995000333308, "expected number of units");



        (uint256 numTokens, ,) = CCI.underwritingStorage(underwriteIdentifier);

        // assert that toAccount get the tokens.
        assertEq(
            Token(token2).balanceOf(toAccount),
            99990000999900000,
            "tokens received"
        );

        assertEq(
            Token(token2).balanceOf(address(CCI)),
            numTokens * (
                CCI.UNDERWRITING_COLLATERAL()
            )/CCI.UNDERWRITING_COLLATERAL_DENOMINATOR(),
            "CCI balance incorrect"
        );

        // Then let the package arrive.
        (bytes memory _metadata, bytes memory toExecuteMessage) = getVerifiedMessage(address(GARP), messageWithContext);

        vm.recordLogs();
        GARP.processPacket(_metadata, toExecuteMessage, FEE_RECIPITANT);
        entries = vm.getRecordedLogs();

        assertEq(
            Token(token2).balanceOf(address(CCI)),
            0,
            "CCI balance not 0"
        );

        assertEq(
            Token(token2).balanceOf(refundTo),
            numTokens + numTokens * (
                CCI.UNDERWRITING_COLLATERAL()
            )/CCI.UNDERWRITING_COLLATERAL_DENOMINATOR(),
            "refundTo balance not expected"
        );

        // Lets execute the message on the source chain and check that the escrow is properly removed.
        (,, messageWithContext) = abi.decode(entries[4].data, (bytes32, bytes, bytes));
        (_metadata, toExecuteMessage) = getVerifiedMessage(address(GARP), messageWithContext);

        // Check for the success event
        vm.expectEmit();
        emit SendAssetSuccess(DESTINATION_IDENTIFIER, convertEVMTo65(toAccount), units, uint256(1e17), address(token1), 1);

        GARP.processPacket(_metadata, toExecuteMessage, FEE_RECIPITANT);
    }
}