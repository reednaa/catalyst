import brownie
import numpy as np
import pytest
from brownie import ZERO_ADDRESS, Token, SwapPool, chain
from brownie.test import given, strategy
from hypothesis import settings
import json, os


@pytest.fixture(autouse=True)
def isolation(module_isolation):
    pass


# POOLNAME = "PS One Two Three"
# POOLSYMBOL = "ps(ott) "
POOLNAME = "PS OneTwoThree"
POOLSYMBOL = "ps(OTT) "


def test_name_length():
    assert len(POOLNAME) <= 16
    assert len(POOLSYMBOL) <= 8


@pytest.fixture(scope="module")
def create_swappool(gov, deploy_swappool, token1, token2, token3):
    def swappool(poolname, poolsymbol):
        tx = deploy_swappool([token1, token2, token3], 2**64, poolname, poolsymbol)
        sp = SwapPool.at(tx.return_value)
        return sp

    yield swappool


def deposit(accounts, token1, token2, token3, sp, gov):

    depositValues = [10 * 10**18, 1000 * 10**18, 1000 * 10**6]

    base_account = accounts[1]
    token1.transfer(base_account, depositValues[0], {"from": gov})
    token2.transfer(base_account, depositValues[1], {"from": gov})
    token3.transfer(base_account, depositValues[2], {"from": gov})

    tokens = [token1, token2, token3]

    for i in range(len(tokens)):
        token = tokens[i]
        depositValue = depositValues[i]

        token.approve(sp, depositValue, {"from": base_account})
        assert token.balanceOf(base_account) == depositValue
        assert token.allowance(base_account, sp) == depositValue

        sp.deposit(token, depositValue, {"from": base_account})

        pt = sp.poolToken(token)
        pt = Token.at(pt)
        assert pt.balanceOf(base_account) == depositValue
        assert token.balanceOf(base_account) == 0

    chain.snapshot()


# NOTE: Test IPC to be removed once Polymerase devnet scripts complete

# create some named fifos for test IPC
def create_fifos(write_pipe, read_pipe):

    if not os.path.exists(write_pipe):
        os.mkfifo(write_pipe)
    if not os.path.exists(read_pipe):
        os.mkfifo(read_pipe)

    write_handle = open(write_pipe, "w")
    read_handle = open(read_pipe, "r")

    return write_handle, read_handle


# write a json message to the other chain
def write_to(obj, f):
    print(json.dumps(obj), file=f)
    f.flush()


# read a json message from other chain
def read_from(f):
    return json.loads(f.readline())


@pytest.mark.no_call_coverage
# @given(swapValue=strategy("uint256", max_value=depositValues[0], min_value=10 ** 18))
def test_multipool_crosschain_swap(
    accounts,
    token1,
    token2,
    token3,
    gov,
    create_swappool,
    polymeraseemulator,
    crosschaininterface,
):

    # When using the polymerase emulator, the target chain should be the same as the
    # sending chain. We get the sending chain using chain.id. However, this does not return
    # the true value when using mainnet_fork. For that reason, we implement the chain_id
    # function in CSSI and get the chain id from that. This will be equal to the connrection
    # created by the catalyst template.
    TARGET_CHAIN_ID = crosschaininterface.chain_id()

    swapValue = 10**18 * 5
    swappoolB = create_swappool("PS B", "PSB")

    # create named pipes to communicate with other chain
    wH, rH = create_fifos(
        "/tmp/test_multichain_swap_a.json", "/tmp/test_multichain_swap_b.json"
    )

    # make initial deposit to swappool
    deposit(accounts, token1, token2, token3, swappoolB, gov)

    base_account = accounts[2]
    assert token2.balanceOf(base_account) == 0

    b = token2.balanceOf(swappoolB)

    # receive swappoolA address and balance
    chainA_data = read_from(rH)
    swappoolA = chainA_data["swappool"]

    # tell chain A about the swappoolB, token2 balance of swappoolB on chainB and the crosschaininterface address and chain id
    # chain A outswap call needs to know this
    write_to(
        {
            "swappool": {
                "address": swappoolB.address,
                "balance": swappoolB.getBalance0(token2),
            },
            "token2": {"balance": token2.balanceOf(swappoolB)},
            "ccsi": {
                "address": crosschaininterface.address,
                "chain_id": TARGET_CHAIN_ID,
            },
        },
        wH,
    )

    # swapToUnits
    # (_chain : uint256, _targetPool : bytes32, _fromAsset : address, _toAsset : uint256, _who : bytes32, _amount : uint256) -> uint256
    tokenArr = [
        swappoolB._tokenIndexing(0),
        swappoolB._tokenIndexing(1),
        swappoolB._tokenIndexing(2),
    ]
    token2AssetIndex = tokenArr.index(token2)

    # logically connect chain B/swappool B to chain A/swappool A. (Not really needed to test swapping from A --> B?)
    swappoolB.createConnection(
        chainA_data["ccsi"]["chain_id"],
        brownie.convert.to_bytes(swappoolA["address"].replace("0x", "")),
        True,
        {"from": gov},
    )

    ###############################################################################
    #   TO DO: this part should eventually be done by polymerase relayers/chain   #
    ###############################################################################

    # read outswap event data
    outswap_event = read_from(rH)
    # {"_data": "0x00000000000000000000000000725dfaf0e481653ab86b2b071027e5daa05ce8b4000000000000000000000000725dfaf0e481653ab86b2b071027e5daa05ce8b40000000000000000000000000063046686e46dc6f15918b61ae2b121458534a5010000000000000000000000000000000038450397365dbb4772afbd29fc780000", "_target": "0x000000000000000000000000e0aa552a10d7ec8760fc6c246d391e698a82ddf9", "_sender": "0xe0aA552A10d7EC8760Fc6c246D391E698a82dDf9"}

    # fakes receiving the IBC packet (chainB IS the target chain)
    polymeraseemulator.call_multichain(
        TARGET_CHAIN_ID,
        outswap_event["_target"],
        outswap_event["_data"],
        {"from": outswap_event["_sender"]},
    )

    # call polymerase contract to execute. Execute triggers "receives" method on the "targetContract" which is the crosschaininterface contract on chainB
    # crosschaininterface:receives() takes the payload data and determines the pool to interact with. It asserts that the pools are connected
    txe = polymeraseemulator.execute(
        polymeraseemulator.lastFilled(), {"from": base_account}
    )
    # Inswap is automatic
    assert token2.balanceOf(base_account) > 0

    # tell chain A about the resulting balances after the swap
    write_to(
        {
            "swappool": {
                "address": swappoolB.address,
                "balance": swappoolB.getBalance0(token2),
            },
            "token2": {"balance": token2.balanceOf(base_account)},
        },
        wH,
    )

    # reset ...
    chain.revert()
