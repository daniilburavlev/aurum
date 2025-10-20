<div align="center">
    <div>
        <h1>XCHG</h1>
    </div>
    <p>Simple Proof-of-Stake blockchain implementation written in Rust for learning purpose</p>
</div>

___

# Getting started
- [Building from sources](#building-from-source)
- [Create new chain](#create-new-chain)
- [Bootstrap node](#bootstrap-node)
- [Connect to existing chain](#connect-to-existing-chain)
- [Wallet](#wallet-client)
  - [Stake](#new-stake)
  - [Transaction](#new-transaction)
## Building from source
### Install rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && \
  rustc --version
```
### Cloning repository
```bash
git clone https://github.com/daniilburavlev/xchg.git && \
  cd xchg
```
### Building with cargo
```bash
cargo build --release
```
> Node & cli wallet also can be installed to PATH
```bash
cargo install --path node
```

# Create new chain
To initialize new blockchain run `node init --storage <PATH_TO_STORAGE> --genesis <PATH_TO_GENESIS_JSON>`
### Genesis JSON file example
```json
[
  {
    "from": "GENESIS",
    "to": "z8TWw9FD1MStBqrvEs5R6E4Gbpenf5tPDKFSK9TdFFax",
    "amount": "2653090",
    "fee": "0",
    "nonce": 1,
    "signature": "GENESIS_8bb67f35694c47a6ae6038f8c0511c42"
  },
  {
    "from": "z8TWw9FD1MStBqrvEs5R6E4Gbpenf5tPDKFSK9TdFFax",
    "to": "STAKE",
    "amount": "100",
    "fee": "0",
    "nonce": 1,
    "signature": "GENESIS_aade396bfc964afdbeee22e5fbdf2c72"
  }
]

```

# Bootstrap node
After initializing chain node can be started by running this command

```bash
xchg run --config config.json
```

Config file example
```json
{
  "http_port": 8080,
  "address": "/ip4/127.0.0.1/tcp/0", // 0 - for random port
  "secret" : "<ECDSA_SECRET>",
  "logs": {
    "dir": "/user/username/.xchg/logs",
    "level": "debug"
  },
  "storage_path": "/user/username/.xchg/storage"
}
```

# Connect to existing chain
You can connect to existing chain by adding nodes' addresses in `nodes` config and run this command
```bash
xchg run --config config.json
```
Known node address example `/ip4/201.168.0.80/tcp/8080/p2p/QmZCprCyPVpERyUB6gPx4wUnSAZAisUfnnEpofkc8yGLXv`

After connecting node will be synced and ready to work

# Wallet
## Creating new wallet
```bash
xchg create-wallet --keystore <PATH>
```

Run `--help` for more info 
## New transaction
To create new transaction run `xchg new-tx --keystore <PATH> --wallet <YOUR_WALLET> --to <WALLET_TO> --amount <AMOUNT> --node <NODE_ADDRESS>`

### Example:

```bash
xchg new-tx --keystore /user/username/.xchg/keystore \
  --node /ip4/201.88.8.122/tcp/9080 \
  --wallet MiAjAWn2QWzai3f1gP1EHASoQor26wpHqPnV1uEmsUK7rKTZNJfKNmuQEEUUFUi25RiGay9pXkAHq6NWMvJvvJQA \
  --to RBm5WNWggU4QqVShre4EUHRcEgAZbvtnZRikhXtVFAp2U8uBLU13Yvm4QxpheN1eBJ26w1SgQ1fGs9cozm9DZGGi \
  --amount 0.001
```

## Staking
Stakes is just transaction sent to STAKE wallet to stake amount in blockchain or UNSTAKE to get back staked amount 

To stake/unstake amount run `xchg new-tx --keystore <PATH> --wallet <YOUR_WALLET> --to <WALLET> --amount <AMOUNT> --node <URL>`
> Note: Only integer amount can be staked
### Example:

```bash
xchg new-tx --keystore /user/username/.xchg/keystore \
  --node http://201.88.8.122:9080 \
  --to STAKE
  --wallet z8TWw9FD1MStBqrvEs5R6E4Gbpenf5tPDKFSK9TdFFax \
  --amount 90
```

# Blocks lookup
Finding blocks by height's index
```bash
xchg find-block --node <URL> --idx 0
```

Response
```json
{
  "idx": 0,
  "validator": "111111111111111111111111111111111",
  "parent_hash": "11111111111111111111111111111111",
  "merkle_root": "7FnvTG9MFvNDyc79e5tZey8KKj7EkAiT5St4fuaL9jyE",
  "txs": [
    {
      "data": {
        "from": "GENESIS",
        "to": "pKgv2HAS9sZJprnNhDahsyoSFvnBQ3E2MStgNyweafLi",
        "amount": "1123344566",
        "fee": "0",
        "nonce": 1,
        "signature": "GENESIS_d5ace3e7a6374c8b81ee09a79576df07"
      },
      "prev_hash": "11111111111111111111111111111111",
      "block": 0,
      "hash": "76ZCaMfEHjFZLHG9yzrwgz1dQpFx44FX9HBZ7FCk3JR5"
    },
    {
      "data": {
        "from": "pKgv2HAS9sZJprnNhDahsyoSFvnBQ3E2MStgNyweafLi",
        "to": "STAKE",
        "amount": "23344566",
        "fee": "0",
        "nonce": 1,
        "signature": "GENESIS_d5ace3e7a6374c8b81ee09a79576df07"
      },
      "prev_hash": "76ZCaMfEHjFZLHG9yzrwgz1dQpFx44FX9HBZ7FCk3JR5",
      "block": 0,
      "hash": "7RbziUv8j32iEyzDx8hggW2kFacKsphTr9vMTYJ8bkDT"
    }
  ],
  "signature": "GENESIS"
}
```