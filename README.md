# NFT Marketplace project
This contract is an example for the NFT marketplace implementation. The contract currently supports 2 token types [PSP34](https://github.com/swanky-dapps/nft) and [RMRK](https://github.com/rmrk-team/rmrk-ink)

### License
Apache 2.0

### 🏗️ How to use - Contracts


##### 💫 Build
- Use this [instructions](https://use.ink/getting-started/setup) to setup your ink!/Rust environment

Clone project
```sh
git clone git@github.com:swanky-dapps/marketplace.git
```

Navigate yourself to marketplace directory
```sh
cd marketplace/contracts/marketplace
```

```sh
cargo contract build
```

##### 💫 Run unit test

```sh
cargo test
```

##### 💫 Run integration test
First start your local node. Recommended the latest [swanky-node](https://github.com/AstarNetwork/swanky-node). After you download and unzip Swanky package for your OS, run it with
```sh
./swanky-node --dev --tmp -lruntime=trace -lruntime::contracts=debug -lerror
```
Navigate to Marketplace project root folder and run the following commands:

```sh
yarn
yarn compile
yarn test
```
##### 💫 Deploy
To manually deploy the contract to local Swanky node or any other node that supports contracts pallet use [Contracts UI](https://contracts-ui.substrate.io/)