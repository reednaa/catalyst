// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.19;

import "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";

import { BaseMultiChainDeployer } from "./BaseMultiChainDeployer.s.sol";
import { JsonContracts } from "./DeployContracts.s.sol";
import { CatalystDescriber } from "../src/registry/CatalystDescriber.sol";
import { CatalystDescriberRegistry } from "../src/registry/CatalystDescriberRegistry.sol";

struct JsonRegistry {
    address describer;
    address describer_registry;
}

contract Registry is BaseMultiChainDeployer {
    using stdJson for string;
    
    bytes32 constant NO_ADDRESS_CODEHASH = 0x0000000000000000000000000000000000000000000000000000000000000000;

    bool get;
    bool verify;

    string config_interfaces;

    JsonRegistry registry;
    JsonContracts contracts;

    address private admin;

    function deploy_describer(bytes32 salt) internal {
        if ((registry.describer.codehash != NO_ADDRESS_CODEHASH) && (get == false)) {
            return;
        }
        address describer = address(new CatalystDescriber{salt: salt}(admin));
        if (verify) require(registry.describer == describer, "not expected address, describer");
        registry.describer = describer;        
    }

    function deploy_registry(bytes32 salt) internal {
        if ((registry.describer_registry.codehash != NO_ADDRESS_CODEHASH) && (get == false)) {
            return;
        }
        address describer_registry = address(new CatalystDescriberRegistry{salt: salt}(admin));
        if (verify) require(registry.describer_registry == describer_registry, "not expected address, describer_registry");
        registry.describer_registry = describer_registry;
    }

    function _deploy() internal {
        load_config();
        deploy_describer(bytes32(0));
        deploy_registry(bytes32(0));

        setRegistry(0);
        setDescriber();
    }

    function deploy() iter_chains(chain_list) broadcast external {
        verify = true;
        admin = vm.envAddress("CATALYST_ADDRESS");
        _deploy();
    }
    function deploy_legacy() iter_chains(chain_list_legacy) broadcast external {
        verify = true;
        admin = vm.envAddress("CATALYST_ADDRESS");
        _deploy();
    }
    function getAddresses() external {
        get = true;
        admin = vm.envAddress("CATALYST_ADDRESS");
        uint256 pk = vm.envUint("CATALYST_DEPLOYER");

        vm.startBroadcast(pk);

        deploy_describer(bytes32(0));
        deploy_registry(bytes32(0));

        vm.stopBroadcast();

        writeToJson();
    }

    function setRegistry(address reg, address describer, uint256 version) iter_chains(chain_list) broadcast external {
        CatalystDescriberRegistry reg = CatalystDescriberRegistry(reg);

        uint256 current_version = reg.catalyst_version();
        require(current_version >= version, "Registry version too far out");
        if (current_version == version) {
            reg.add_describer(describer);
        } else if (current_version == version + 1) {
            reg.modify_describer(describer, version);
        } else {
            require(false, "not implemented");
        }
    }

    function setRegistryLegacy(address reg, address describer, uint256 version) iter_chains(chain_list_legacy) broadcast external {
        CatalystDescriberRegistry reg = CatalystDescriberRegistry(reg);

        uint256 current_version = reg.catalyst_version();
        require(current_version >= version, "Registry version too far out");
        if (current_version == version) {
            reg.add_describer(describer);
        } else if (current_version == version + 1) {
            reg.modify_describer(describer, version);
        } else {
            require(false, "not implemented");
        }

    }

    function setRegistry(uint256 version) internal {
        CatalystDescriberRegistry reg = CatalystDescriberRegistry(registry.describer_registry);

        uint256 current_version = reg.catalyst_version();
        require(current_version >= version, "Registry version too far out");
        if (current_version == version) {
            reg.add_describer(registry.describer);
        } else if (current_version == version + 1) {
            reg.modify_describer(registry.describer, version);
        } else {
            require(false, "not implemented");
        }
    }

    function setDescriber() internal {
        CatalystDescriber desc = CatalystDescriber(registry.describer);
        // Set (or update) the templates
        address current_volatile_template = desc.version_to_template("volatile");
        if (current_volatile_template != contracts.volatile_template) desc.modifyWhitelistedTemplate(contracts.volatile_template, "volatile");

        address current_amplified_template = desc.version_to_template("amplified");
        if (current_amplified_template != contracts.amplified_template) desc.modifyWhitelistedTemplate(contracts.amplified_template, "amplified");

        // Set (or update) the cross-chain interfaces
        string[] memory availableInterfaces = vm.parseJsonKeys(config_interfaces, string.concat(".", rpc[chain]));
        for (uint256 i = 0; i < availableInterfaces.length; ++i) {
            string memory incentiveVersion = availableInterfaces[i];
            address excepted_cci = abi.decode(config_interfaces.parseRaw(string.concat(".", rpc[chain], ".", incentiveVersion, ".interface")), (address));
            
            address current_cci = desc.version_to_cci(incentiveVersion);
            if (current_cci != excepted_cci) desc.modifyWhitelistedCCI(excepted_cci, incentiveVersion);
        }

        // Set (or update) the factory.
        address current_factory = desc.version_to_factory("v1");
        if (current_factory != contracts.factory) desc.modifyWhitelistedFactory(contracts.factory, "v1");
    }


    function load_config() internal {
        string memory pathRoot = vm.projectRoot();
        string memory pathToContractConfig = string.concat(pathRoot, "/script/config/config_contracts.json");
        string memory pathToInterfacesConfig = string.concat(pathRoot, "/script/config/config_interfaces.json");
        config_interfaces = vm.readFile(pathToInterfacesConfig);

        // Get the contracts
        string memory config_contract = vm.readFile(pathToContractConfig);
        registry = abi.decode(config_contract.parseRaw(string.concat(".registry")), (JsonRegistry));
        contracts = abi.decode(config_contract.parseRaw(string.concat(".contracts")), (JsonContracts));

    }

    function writeToJson() internal {
        string memory pathRoot = vm.projectRoot();
        string memory pathToContractConfig = string.concat(pathRoot, "/script/config/config_contracts.json");
        string memory obj = "";

        vm.serializeAddress(obj, "describer", registry.describer);
        string memory finalJson = vm.serializeAddress(obj, "describer_registry", registry.describer_registry);

        vm.writeJson(finalJson, pathToContractConfig, string.concat(".registry"));
    }
}

