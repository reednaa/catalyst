// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import { TestCommon } from "../TestCommon.t.sol";
import {Token} from "../mocks/token.sol";
import "../../src/ICatalystV1Vault.sol";


contract TestSwapIntegration is TestCommon {

    event Message(
        bytes32 destinationIdentifier,
        bytes recipitent,
        bytes message
    );

    event SendAsset(
        bytes32 channelId,
        bytes toVault,
        bytes toAccount,
        address fromAsset,
        uint8 toAssetIndex,
        uint256 fromAmount,
        uint256 minOut,
        uint256 Units,
        uint256 fee
    );

    bytes32 FEE_RECIPITANT = bytes32(uint256(uint160(9191919191)));

    address TO_ACCOUNT = address(uint160(123123));

    address _REFUND_GAS_TO = TO_ACCOUNT;

    IncentiveDescription _INCENTIVE = IncentiveDescription({
        maxGasDelivery: 1199199,
        maxGasAck: 1188188,
        refundGasTo: _REFUND_GAS_TO,
        priceOfDeliveryGas: 123321,
        priceOfAckGas: 321123,
        targetDelta: 30 minutes
    });

    function _getTotalIncentive(IncentiveDescription memory incentive) internal pure returns(uint256) {
        return incentive.maxGasDelivery * incentive.priceOfDeliveryGas + incentive.maxGasAck * incentive.priceOfAckGas;
    }

    function pool1() internal returns(address vault1, address vault2) {

        // Deploy tokens.
        address[] memory tokens1 = getTokens(3);
        address[] memory tokens2 = getTokens(1);
        approveTokens(address(catFactory), tokens1);
        approveTokens(address(catFactory), tokens2);

        // Deploy a volatile vault
        uint256[] memory amounts1 = new uint256[](3);
        amounts1[0] = 100*10**18; amounts1[1] = 200*10**18; amounts1[2] = 300*10**18;
        uint256[] memory weights1 = new uint256[](3);
        weights1[0] = 1; weights1[1] = 1; weights1[2] = 1;
        vault1 = deployVault(
            tokens1,
            amounts1,
            weights1,
            10**18,
            0
        );

        // Deploy a volatile vault
        uint256[] memory amounts2 = new uint256[](1);
        amounts2[0] = 100*10**18;
        uint256[] memory weights2 = new uint256[](1);
        weights2[0] = 1;
        vault2 = deployVault(
            tokens2,
            amounts2,
            weights2,
            10**18,
            0
        );
        
        setConnection(vault1, vault2, DESTINATION_IDENTIFIER, DESTINATION_IDENTIFIER);
    }

    function pool2() internal returns(address vault1, address vault2) {
        // Deploy tokens. 
        address[] memory tokens = getTokens(3);
        approveTokens(address(catFactory), tokens);

        // Deploy an amplified vault
        uint256[] memory amounts = new uint256[](3);
        amounts[0] = 100*10**18; amounts[1] = 200*10**18; amounts[2] = 300*10**18;
        uint256[] memory weights = new uint256[](3);
        weights[0] = 1; weights[1] = 1; weights[2] = 1;
        amounts[0] = 100*10**18; amounts[1] = 200*10**18; amounts[2] = 300*10**18;
        weights[0] = 1; weights[1] = 1; weights[2] = 1;
        vault1 = deployVault(
            tokens,
            amounts,
            weights,
            10**18 / 2,
            0
        );

        vault2 = vault1;

        setConnection(vault1, vault2, DESTINATION_IDENTIFIER, DESTINATION_IDENTIFIER);
    }

    function test_cross_chain_swap_volatile() external {
        uint256 amount = 10**18*1e4;
        (address vault1, address vault2) = pool1();
        t_cross_chain_swap(vault1, vault2, amount);
    }
    
    function t_cross_chain_swap(address fromVault, address toVault, uint256 amount) internal {
        address tkn = ICatalystV1Vault(fromVault)._tokenIndexing(0);

        Token(tkn).approve(fromVault, amount);

        // TODO: this message, we need to add the units.
        // vm.expectEmit();
        // emit Message(
        //     CHANNEL_ID,
        //     hex"0000000000000000000000005991a2df15a8f6a256d3ec51e99254cd3fb576a9",
        //     hex"80000000000000000000000000000000000000000000000000000000001231230000000000000000000000000000000000000000000000000000000000000539003fd017ce8d9e2f46a0d62e4cb993736c47339aad1b29a35c05e653dd3964d4e9140000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c7183455a4c133ae270771860664b6b7ec320bb1140000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c7183455a4c133ae270771860664b6b7ec320bb1000000124c5f00140000000000000000000000000000000000000000000000000000000000000000000000000000000000000000104fbc016f4bb334d775a19e8a6510109ac63e00140000000000000000000000000000000000000000000000000000000000000000000000000000000000000000037eda3adb1198021a9b2e88c22b464fd38db3f3140000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001e0f3000000000000000000000000000000000000000000000000400c33a879f0c1ba000000000000000000000000000000000000000000000000000000000000000c8d00000000000000000000000000000000000000000000021e19e0c9bab2400000140000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0cb889707d426a7a386870a03bc70d1b0697598000000010000"
        // );

        uint256 MINOUT = 3213;
        
        uint256 snapshotId = vm.snapshot();
        uint256 UNITS = ICatalystV1Vault(fromVault).sendAsset{value: _getTotalIncentive(_INCENTIVE)}(
            DESTINATION_IDENTIFIER,
            convertEVMTo65(toVault),
            convertEVMTo65(TO_ACCOUNT),
            tkn,
            0,
            amount,
            MINOUT,
            TO_ACCOUNT,
            _INCENTIVE
        );
        vm.revertTo(snapshotId);


        vm.expectEmit();
        emit SendAsset(
            DESTINATION_IDENTIFIER,
            convertEVMTo65(toVault),
            convertEVMTo65(TO_ACCOUNT),
            tkn,
            0,
            amount,
            MINOUT,
            UNITS,
            0
        );

        vm.expectCall(
            address(CCI),
            _getTotalIncentive(_INCENTIVE), // value
            abi.encodeCall(
                CCI.sendCrossChainAsset,
                (
                    DESTINATION_IDENTIFIER,
                    convertEVMTo65(toVault),
                    convertEVMTo65(TO_ACCOUNT),
                    0,
                    UNITS,
                    MINOUT,
                    amount,
                    tkn,
                    _INCENTIVE,
                    hex""
                )
            )
        );

        vm.recordLogs();
        ICatalystV1Vault(fromVault).sendAsset{value: _getTotalIncentive(_INCENTIVE)}(
            DESTINATION_IDENTIFIER,
            convertEVMTo65(toVault),
            convertEVMTo65(TO_ACCOUNT),
            tkn,
            0,
            amount,
            MINOUT,
            TO_ACCOUNT,
            _INCENTIVE
        );  

        // The message is event 2. (index 1)
        Vm.Log[] memory entries = vm.getRecordedLogs();

        (bytes32 destinationIdentifier, bytes memory recipitent, bytes memory messageWithContext) = abi.decode(entries[1].data, (bytes32, bytes, bytes));

        (bytes memory _metadata, bytes memory toExecuteMessage) = getVerifiedMessage(address(GARP), messageWithContext);

        GARP.processMessage(_metadata, toExecuteMessage, FEE_RECIPITANT);
    }
}