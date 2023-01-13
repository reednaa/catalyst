import pytest
from brownie import ZERO_ADDRESS


@pytest.fixture(scope="module")
def deploy_pool(accounts, swap_factory, cross_chain_interface, swap_pool_class, deployer):
    def _deploy_pool(
        tokens,
        token_balances,
        weights,
        amp,
        name,
        symbol,
        deployer = deployer,
        only_local = False,
        template_index = None,
        swap_pool_class = swap_pool_class
    ):
        for i, token in enumerate(tokens):
            token.transfer(deployer, token_balances[i], {"from": accounts[0]})
            token.approve(swap_factory, token_balances[i], {"from": deployer})

        if template_index is None:
            template_index = 0 if swap_pool_class == "volatile" else 1

        tx = swap_factory.deploy_swappool(
            template_index,
            tokens,
            token_balances,
            weights,
            amp,
            name,
            symbol,
            ZERO_ADDRESS if only_local else cross_chain_interface,
            {"from": deployer},
        )

        return swap_pool_class.at(tx.return_value)

    yield _deploy_pool



@pytest.fixture(scope="module")
def max_pool_assets():
    return 3


@pytest.fixture(scope="module")
def amplification(request, raw_config, swap_pool_type):

    if swap_pool_type == "volatile":
        yield None

    elif swap_pool_type == "amplified":

        # NOTE: the --amplification flag overrides the amplification value set on the config file if present
        amplification = request.config.getoption("--amplification") or raw_config["amplification"]
        amplification = eval(amplification)     # Parse expressions such as '10**18'

        assert amplification < 10**18 and amplification > 0

        yield amplification



# 'group_' fixtures
# Each of these expose info on ALL the pools defined on the loaded test config file
# (i.e. they ARE NOT parametrized for every single pool defined on the test config file)

@pytest.fixture(scope="module")
def group_config(raw_config, amplification, max_pool_assets):

    raw_pools_config = raw_config["pools"]

    # Verify the pools config
    for config in raw_pools_config:
        _verify_pool_config(config, max_pool_assets)

    # Inject the amplification value into each pool config object
    yield [
        {
            "tokens"        : config["tokens"],
            "init_balances" : [eval(balance) for balance in config["initBalances"]],      # Evaluate balance expressions (e.g. '10**18')
            "weights"       : config["weights"],
            "name"          : config["name"],
            "symbol"        : config["symbol"],
            "amplification" : amplification
        } for config in raw_pools_config
    ]


@pytest.fixture(scope="module")
def group_tokens(group_config, tokens):
    yield [[tokens[i] for i in pool["tokens"]] for pool in group_config]


@pytest.fixture(scope="module")
def group_pools(group_config, group_tokens, deploy_pool, deployer):

    yield [
        deploy_pool(
            tokens         = tokens,
            token_balances = pool["init_balances"],
            weights        = pool["weights"],
            amp            = pool["amplification"] if pool["amplification"] is not None else 10**18,
            name           = pool["name"],
            symbol         = pool["nymbol"],
            deployer       = deployer,
        ) for pool, tokens in zip(group_config, group_tokens)
    ]



# 'pool_' fixtures
# Each of these expose info on a SINGLE pool for each pool defined on the loaded test config file
# (i.e. they ARE parametrized for every single pool defined on the test config file)

@pytest.fixture(scope="module")
def pool_config(raw_pool_config, amplification, max_pool_assets):

    _verify_pool_config(raw_pool_config, max_pool_assets)

    yield {
        "tokens"        : raw_pool_config["tokens"],
        "init_balances" : [eval(balance) for balance in raw_pool_config["initBalances"]],      # Evaluate balance expressions (e.g. '10**18')
        "weights"       : raw_pool_config["weights"],
        "name"          : raw_pool_config["name"],
        "symbol"        : raw_pool_config["symbol"],
        "amplification" : amplification
    }


@pytest.fixture(scope="module")
def pool_tokens(pool_config, tokens):
    yield [tokens[i] for i in pool_config["tokens"]]


@pytest.fixture(scope="module")
def pool(pool_config, pool_tokens, deploy_pool, deployer):

    yield deploy_pool(
            tokens         = pool_tokens,
            token_balances = pool_config["init_balances"],
            weights        = pool_config["weights"],
            amp            = pool_config["amplification"] if pool_config["amplification"] is not None else 10**18,
            name           = pool_config["name"],
            symbol         = pool_config["symbol"],
            deployer       = deployer,
        )



def _verify_pool_config(config, max_pool_assets):
    assert "tokens" in config and len(config["tokens"]) > 0 and len(config["tokens"]) <= max_pool_assets
    assert "initBalances" in config and len(config["initBalances"]) == len(config["tokens"])
    assert "weights" in config and len(config["weights"]) == len(config["tokens"])
    assert "name" in config and isinstance(config["name"], str)
    assert "symbol" in config and isinstance(config["symbol"], str)