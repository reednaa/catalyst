import brownie
import json
from .collectNetworkInformation import BSCCONFIG, modifyConfig, stringToHex
from brownie import ZERO_ADDRESS


def main():
    dispatch = BSCCONFIG["dispatcher"]["address"]
    assert brownie.web3.eth.get_code(dispatch) != brownie.web3.eth.get_code(
        ZERO_ADDRESS
    ), "Dispatch is incorrect, (EVM ERROR, WRONG DISPATCH)"
    acct = brownie.accounts.from_mnemonic(
        BSCCONFIG["accounts"][0]["mnemonic"]["phrase"]
    )
    assert acct.balance() > 0, "Account is not funded (EVM ERROR, WRONG KEY)"
    from scripts.deployCatalyst import Catalyst  # noqa: E402

    ps = Catalyst(
        acct, poolname="psETHEREUM", poolsymbol="psETH", ibcinterface=dispatch
    )
    ps.crosschaininterface.registerPort({"from": acct})
    ch0 = brownie.convert.datatypes.HexString(
        stringToHex("channel-0"),
        type_str="bytes32",
    )
    ps.crosschaininterface.setChannelForChain(1234, ch0, {"from": acct})

    modifyConfig(["bsc", "swappool"], ps.swappool.address)
    modifyConfig(["bsc", "token0"], ps.tokens[0].address)
