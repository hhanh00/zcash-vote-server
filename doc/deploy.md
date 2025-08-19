# Building

Ubuntu Pre-reqs

```
sudo apt install build-essential libssl-dev pkg-config
```

Build with Cargo.

```
cargo b -r
```

# Deployment

`zcash-vote-server` is an blockchain application that uses the
CometBFT ABCI.

In development, you can run it with a single node but in production,
it should be deployed with multiple validators.

With a single node, the election can be compromised if the
authority that runs the server is malicious. For instance,
if they collude with the election creator and have the
ballot decryption key, they can selectively choose to
include or exclude ballots to swerve the results any way
they want[^1].

To eliminate this risk, we deploy a blockchain running
the [Comet BFT consensus engine](https://docs.cometbft.com/v1.0/).

There is a minimum of *four* validators[^2].

The election is secured as long as 2/3 of the validators are honest.

## ACBI

The voting app connects to the vote server (using REST). The vote
server submits the vote to the CometBFT engine (`cometbft`) using
the RPC API. The vote goes through the consensus workflow. It gets
checked by the vote server, and put into a block in the voting
blockchain.

Once the block is finalized, the vote server commits the vote
to its database.

- There can be separate voting chains per election. But we can
form groups of voting servers that manage multiple elections.
- Voters can submit their vote to any node including their own
node if they want. However, the consensus is decided by the
validators[^3].

## Development & Single Node Testing

- Install the `cometbft` server from their release page.
- Initialize a new chain: `cometbft init`
- By default, the BFT port is 26658. It is the value
set in `Rocket.toml` too.
- Start `cometbft node`
- Start `zcash-vote-server`

They should pair up and you should see blocks being produced every second.

### Reset

If you want to reset the system and delete every vote, do the following:
- Stop both `cometbft` and `zcash-vote-server`
- Delete `vote.db` (or whatever database name you set in `Rocket.toml`)
- Delete the blockchain data: `cometbft --unsafe-reset-all`

## Deployment in Production

In production, you will need at least 4 validators to allow for
the possibility of 1 validator failing.

There are guides for this setup in the CometBFT website. In summary,
the steps are:
- Initialize a blockchain: `cometbft init`
- This creates a `genesis.json`, `config.toml` and `priv_validator_key.json`
in the `.cometbft/config` directory
- Repeat on every validator node
- At this point, every node has its own genesis data. We want them to
be on the same blockchain, therefore we need to have the same `genesis.json`.
- Before we copy it over, we have to edit its set of validators.
    - Currently, "validators" has a single entry

```json
    {
      "address": "911242A4791C7734286595FCD472F2F511C31B59",
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value": "frS/SVRYiveGUr0zV/TUD8V7jY6koH9pH3q2eLGB55w="
      },
      ...
    }
```

- You need to go to the other nodes and copy the section "address", "pub_key"
from the `priv_validator_key.json` file. If the validators are under
different authorities (as it should), you need to ask them to provide you
this info.
- Then you can distribute the `genesis.json` file

- The entry `persistent_peers` of `config.toml` must be updated too.
    - Ask each validator operator to run `cometbft show-node-id`
    and give you the node id. The node id is `<nodeid>@<ip>:<port>`
    for example: `e443d4e7f690a0dacf1c0308f6db644929f415c1@www.example.com:26658`
    It must be the external IP if the validator is running behind a NAT.
    - persistent_peers is a comma delimited list of peers.
    - Do not include your *own* peer address

Once the validators (and their respective `zcash-vote-server`) runs,
you should see blocks being created.

## Single machine cluster setup

Option 1 is to use docker. I haven't tried but it may be the easiest
way.

Option 2 is to run 4 pairs of `cometbft/zcash-vote-server` locally.
This is possible but since ports must be unique, it requires more
configuration.

- You need different directories for `cometbft`. You need to add
the argument `--home <nodedir>` to every `cometbft` command.
For example, to initialize the directory, it becomes
`cometbft init --home ~/.cometbft/n1`
- You need to update the `config.toml` ports for ABCI, RPC and P2P
By default, they are 26658, 26657, 26656. Each node must have a
different set. For example, node 1 could have 20002, 20001, 20000
and node 2 could have 20012, 20011, 20010.
- Set `allow_duplicate_ip` to **true** otherwise nodes will deny
more than one connection for the same IP.
- Finally, you also need to create different `zcash-vote-server`
configuration based on `Rocket.toml`. You want to
change `port`, `db_path` and `cometbft_port` to avoid conflicts.
- Then to run with a new config file: `ROCKET_CONFIG=node1.toml zcash-vote-server`

[^1]: Thanks to Least Authority Audit for reporting the issue.
[^2]: Validators are also nodes. Nodes do not have to be validators.
[^3]: It's a Proof of Authority blockchain.
