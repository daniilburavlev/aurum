<div align="center">
    <div>
        <h1>Aurum</h1>
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
git clone https://github.com/daniilburavlev/aurum.git && \
  cd aurum
```
### Building with cargo
```bash
cargo build 
```
> Node & cli wallet also can be installed to PATH
```bash
cargo install --path node && \
cargo install --path wallet-cli
```

# Create new chain
To initialize new blockchain run `node init --path <PATH_TO_GENESIS_JSON>`
### Genesis JSON file example
```json
[
  {
    "hash": "GENESIS_519c2c360cbd4f0a8b6842fa9556b9e0",
    "from": "GENESIS",
    "to": "031ff76636df08470e4bf11b3108a03ca504a7732ab9e9cadad1cc2f63f6944bd1",
    "amount": "2653090",
    "nonce": 1,
    "timestamp": 1009227600,
    "signature": "GENESIS",
    "block": 0
  },
  {
    "hash": "GENESIS_03fddab9872e40d2b1a7076f1f29b4fa",
    "from": "031ff76636df08470e4bf11b3108a03ca504a7732ab9e9cadad1cc2f63f6944bd1",
    "to": "STAKE",
    "amount": "1323090",
    "nonce": 1,
    "timestamp": 1009227600,
    "signature": "GENESIS",
    "block": 0
  }
]
```

# Bootstrap node
After initializing chain node can be started by running this command
```bash
aurum --wallet <WALLET>
```
New wallet can be created by `wallet-cli create` or running `aurum` without `--wallet` argument
```
Wallet is not specified, create new one? [y/n]: 
```

# Connect to existing chain
You can connect to existing chain by running this command localy
```bash
aurum --nodes <KNOWN_NODES> --wallet <LOCAL_WALLET>
```
Known node address example `/ip4/201.168.0.80/tcp/8080/QmZCprCyPVpERyUB6gPx4wUnSAZAisUfnnEpofkc8yGLXv`

After connecting node will be synced and ready to work

# Wallet client
## New stake
To stake amount run `wallet-cli stake --from <YOUR_WALLET> --amount <AMOUNT> --node <NODE_ADDRESS>`
> Note: Only integer amount can be staked
### Example:

```bash
wallet-cli \
stake --node /ip4/201.88.8.122/tcp/9080 \
--from 031ff76636df08470e4bf11b3108a03ca504a7732ab9e9cadad1cc2f63f6944bd1 \
--amount 90
```
## New transaction
To create new transaction run `wallet-cli tx --from <YOUR_WALLET> --to <WALLET_TO> --amount <AMOUNT> --node <NODE_ADDRESS>`

### Example:

```bash
wallet-cli \
tx --node /ip4/201.88.8.122/tcp/9080 \
--from 031ff76636df08470e4bf11b3108a03ca504a7732ab9e9cadad1cc2f63f6944bd1 \
--to 031ab26cdb3d1907e5e9fc3fc4e96ec3df41bfcdc1dc8e50ec43d37163c27014f5 \
--amount 0.001
```