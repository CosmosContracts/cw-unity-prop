# Verifying the Unity Prop Smart Contract

The source code for the Unity prop smart contract is available in the [cw-unity-prop repo](https://github.com/CosmosContracts/cw-unity-prop).

To verify the code deployed on chain you will need `docker` and `junod` installed on your machine.

### Compile contracts locally

Clone the repo and enter the directory:
```bash
git clone https://github.com/CosmosContracts/cw-unity-prop
cd cw-unity-prop
git checkout v0.3.0
```

Compile the contracts:
```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6
```

This should output info about the contract sha256 hash:
```bash
Optimizing cw_unity_prop.wasm ...
Creating hashes ...
3481e613b412706204124ce081ac9cba18eb044d5c78c922e55e162e411d4b47  cw_unity_prop.wasm
Info: sccache stats after build
```

Make note of this for later.

### Verifying the contract code on chain

The contract has already been deployed on chain. You can [view it on MintScan here](https://www.mintscan.io/juno/wasm/contract/juno1nz96hjc926e6a74gyvkwvtt0qhu22wx049c6ph6f4q8kp3ffm9xq5938mr).

In the `v4.0.0` `junod` tag you can find the address hard coded in the code [here](https://github.com/CosmosContracts/juno/blob/299fe4bdee7a7a8b45cd2776359243fdf3630e5a/app/upgrade/upgrade_handler.go#L21):
```go
// UnityContractByteAddress is the bytes of the public key for the address of the Unity contract
// $ junod keys parse juno1nz96hjc926e6a74gyvkwvtt0qhu22wx049c6ph6f4q8kp3ffm9xq5938mr
// human: juno
// bytes: 5BEF9E5318ED6716A11179C70B06656E9FB91D241A1C594F344B325D9110D94C
const UnityContractByteAddress = "5BEF9E5318ED6716A11179C70B06656E9FB91D241A1C594F344B325D9110D94C"
```

You can query info about this contract on chain:
```bash
junod q wasm contract juno1nz96hjc926e6a74gyvkwvtt0qhu22wx049c6ph6f4q8kp3ffm9xq5938mr
```

This should return:
```bash
address: juno1nz96hjc926e6a74gyvkwvtt0qhu22wx049c6ph6f4q8kp3ffm9xq5938mr
contract_info:
  admin: ""
  code_id: "253"
  created: null
  creator: juno1cdqa6wd7fyxx4nq3fh630zsvjrwqj9tgea6npm
  extension: null
  ibc_port_id: ""
  label: Juno Unity Prop v0.3.0
```

Note the `code_id` `253`, now lets download the wasm binary for that code:
```bash
junod q wasm code 253 cw_unity_prop.wasm
```

Get the hash and not that it matches the checksums in [v0.3.0](https://github.com/CosmosContracts/cw-unity-prop/releases/tag/v0.3.0):
```bash
shasum -a 256 cw_unity_prop.wasm
```

It should output:
```bash
3481e613b412706204124ce081ac9cba18eb044d5c78c922e55e162e411d4b47  cw_unity_prop.wasm
```

#### More info on verifying smart contracts
- https://docs.cosmwasm.com/docs/1.0/smart-contracts/verify
- https://secdao.medium.com/smart-contract-verification-75f9a7e7f23
