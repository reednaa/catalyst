// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import { Permit2 } from "../lib/permit2/src/Permit2.sol";

// Math libs
import { CatalystMathVol } from "../src/registry/CatalystMathVol.sol";
import { CatalystMathAmp } from "../src/registry/CatalystMathAmp.sol";

// Registry
import { CatalystDescriber } from "../src/registry/CatalystDescriber.sol";
import { CatalystDescriberRegistry } from "../src/registry/CatalystDescriberRegistry.sol";

// Router
import { CatalystRouter } from "../src/router/CatalystRouter.sol";
import { RouterParameters } from "../src/router/base/RouterImmutables.sol";

// Core Catalyst
import { CatalystFactory } from "../src/CatalystFactory.sol";
import { CatalystGARPInterface } from "../src/CatalystGARPInterface.sol";
/// Catalyst Templates
import { CatalystVaultVolatile } from "../src/CatalystVaultVolatile.sol";
import { CatalystVaultAmplified } from "../src/CatalystVaultAmplified.sol";


// Generalised Incentives
import { IncentivizedMockEscrow } from "GeneralisedIncentives/src/apps/mock/IncentivizedMockEscrow.sol";

struct JsonContracts {
    address amplified_mathlib;
    address amplified_template;
    address crosschaininterface;
    address describer;
    address describer_registry;
    address factory;
    address permit2;
    address router;
    address volatile_mathlib;
    address volatile_template;
}

contract DeployCatalyst is Script {
    using stdJson for string;

    // string config_contract;
    // string config_chain;

    bool fillDescriber = false;

    JsonContracts contracts;

    bytes32 chainIdentifier;

    error NoWrappedGasTokenFound();

    function getOrDeployPermit2() internal returns(address permit2) {
        permit2 = contracts.permit2;
        if (permit2 != address(0)) return permit2;

        permit2 = address(new Permit2());
    }

    function getGasToken() internal returns(address wrappedGas) {
        wrappedGas = address(0); // TODO:

        if (wrappedGas == address(0)) {
            revert NoWrappedGasTokenFound();
        }
    }

    function whitelistAllCCIs(CatalystDescriber catalyst_describer) internal {

    }

    function deployAllContracts() internal {

        // Deploy Registry
        
        CatalystDescriber catalyst_describer = CatalystDescriber(contracts.describer);
        if (address(catalyst_describer) == address(0)) {
            catalyst_describer = new CatalystDescriber();
        }
        contracts.describer = address(catalyst_describer);

        {
            CatalystDescriberRegistry describer_registry = CatalystDescriberRegistry(contracts.describer_registry); 
            if (address(describer_registry) == address(0)) {
                describer_registry = new CatalystDescriberRegistry();
                fillDescriber = true;
            }
            contracts.describer_registry = address(describer_registry);
        }

        // Deploy Factory
        CatalystFactory factory = CatalystFactory(contracts.factory);
        if (address(factory) == address(0)) {
            factory = new CatalystFactory(0);
        }
        contracts.factory = address(factory);

        // Deploy Templates
        address volatile_mathlib = contracts.volatile_mathlib;
        if (volatile_mathlib == address(0)) volatile_mathlib = address(new CatalystMathVol());
        contracts.volatile_mathlib = address(volatile_mathlib);

        address volatile_template = contracts.volatile_template;
        if (volatile_template == address(0)) {
            volatile_template = address(
                new CatalystVaultAmplified(address(factory), volatile_mathlib)
            );
        }
        contracts.volatile_template = address(volatile_template);

        address amplified_mathlib = contracts.amplified_mathlib;
        if (amplified_mathlib == address(0)) amplified_mathlib = address(new CatalystMathAmp());
        contracts.amplified_mathlib = address(amplified_mathlib);

        address amplified_template = contracts.amplified_template;
        if (amplified_template == address(0)) {
            amplified_template = address(
                new CatalystVaultAmplified(address(factory), amplified_mathlib)
            );
        }
        contracts.amplified_template = address(amplified_template);

        // Permit2 for router
        address permit2 = getOrDeployPermit2();

        // Get the wrapped token for router
        address wrappedGas = getGasToken();

        // Router
        CatalystRouter router = CatalystRouter(payable(contracts.router));
        if (address(router) == address(0)) {
            router = new CatalystRouter(RouterParameters({
                permit2: address(permit2),
                weth9: address(wrappedGas)
            }));
        }
        contracts.router = address(router);
    }

    function setupDescriber() internal {
        CatalystDescriber catalyst_describer = CatalystDescriber(contracts.describer);

        catalyst_describer.add_vault_factory(contracts.factory);
        catalyst_describer.add_whitelisted_template(contracts.volatile_template, 1);
        catalyst_describer.add_whitelisted_template(contracts.amplified_template, 1);
        catalyst_describer.add_whitelisted_cci(contracts.crosschaininterface);
    }


    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_KEY");

        string memory pathRoot = vm.projectRoot();
        string memory pathToChainConfig = string.concat(pathRoot, "/config/config_chain.json");
        string memory pathToContractConfig = string.concat(pathRoot, "/config/config_contracts.json");

        string memory chain = vm.envString("CHAIN_NAME");
        string memory config_chain = vm.readFile(pathToChainConfig);
        chainIdentifier = bytes32(vm.parseJsonUint(config_chain, chain));

        string memory config_contract = vm.readFile(pathToContractConfig);
        contracts = abi.decode(vm.parseJson(config_contract, chain), (JsonContracts));

        vm.startBroadcast(deployerPrivateKey);

        deployAllContracts();

        vm.stopBroadcast();

        uint256 registryPrivateKey = vm.envUint("REGISTRY_KEY");

        // Fill registry
        if (fillDescriber == true) {
            vm.startBroadcast(registryPrivateKey);
            setupDescriber();
            vm.stopBroadcast();
        }

        // Save json

        string memory obj = chain;

        vm.serializeAddress(obj, "amplified_mathlib", contracts.amplified_mathlib);
        vm.serializeAddress(obj, "amplified_template", contracts.amplified_template);
        vm.serializeAddress(obj, "crosschaininterface", contracts.crosschaininterface);
        vm.serializeAddress(obj, "describer", contracts.describer);
        vm.serializeAddress(obj, "describer_registry", contracts.describer_registry);
        vm.serializeAddress(obj, "factory", contracts.factory);
        vm.serializeAddress(obj, "permit2", contracts.permit2);
        vm.serializeAddress(obj, "router", contracts.router);
        vm.serializeAddress(obj, "volatile_mathlib", contracts.volatile_mathlib);
        vm.serializeAddress(obj, "volatile_template", contracts.volatile_template);
        
        // string memory finalJson = vm.serializeString(chain, "object", output);

        vm.writeJson(obj, "./config/config_contracts.json", chain);

    }
}
