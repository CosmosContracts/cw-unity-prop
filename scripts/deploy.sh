#!/bin/bash

if [ "$1" = "" ]
then
  echo "Usage: $0 1 arg required - juno address"
  exit
fi

IMAGE_TAG=${2:-"v6.0.0-alpha"}
CONTAINER_NAME="juno_cw_unity_prop"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
CHAIN_ID='testing'
RPC='http://localhost:26657/'
TXFLAG="--gas-prices 0.1$DENOM --gas auto --gas-adjustment 1.3 -y -b block --chain-id $CHAIN_ID --node $RPC"
BLOCK_GAS_LIMIT=${GAS_LIMIT:-100000000} # mirrors mainnet

echo "Building $IMAGE_TAG"
echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

# orphans
docker kill $CONTAINER_NAME
docker volume rm -f junod_data

# run junod docker
docker run --rm -d --name $CONTAINER_NAME \
    -e STAKE_TOKEN=$DENOM \
    -e GAS_LIMIT="$GAS_LIMIT" \
    -e UNSAFE_CORS=true \
    -p 1317:1317 -p 26656:26656 -p 26657:26657 \
    --mount type=volume,source=junod_data,target=/root \
    ghcr.io/cosmoscontracts/juno:$IMAGE_TAG /opt/setup_and_run.sh $1

# compile
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6

# copy wasm to docker container
docker cp artifacts/cw_unity_prop.wasm $CONTAINER_NAME:/cw_unity_prop.wasm

# validator addr
VALIDATOR_ADDR=$($BINARY keys show validator --address)
echo "Validator address:"
echo $VALIDATOR_ADDR

BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR)
echo "Pre-store balance:"
echo $BALANCE_1

echo "Address to deploy contracts: $1"
echo "TX Flags: $TXFLAG"

# errors from this point on are no bueno
set -e

# upload wasm
CONTRACT_CODE=$($BINARY tx wasm store "/cw_unity_prop.wasm" --from validator $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')
echo "Stored: $CONTRACT_CODE"

BALANCE_2=$($BINARY q bank balances $VALIDATOR_ADDR)
echo "Post-store balance:"
echo $BALANCE_2

# provision juno default user i.e. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
echo "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose" | $BINARY keys add test-user --recover --keyring-backend test

# instantiate
INIT='{
  "native_denom": "'"$DENOM"'",
  "withdraw_address": "'"$1"'",
  "withdraw_delay_in_days": 28
}'
echo "$INIT" | jq .

# --no-admin sent in test
$BINARY tx wasm instantiate $CONTRACT_CODE "$INIT" --from "validator" --label "juno unity prop" $TXFLAG --no-admin 
RES=$?

# get contract addr
CONTRACT_ADDRESS=$($BINARY q wasm list-contract-by-code $CONTRACT_CODE --output json | jq -r '.contracts[-1]')

# attempt to trigger withdrawal
START_WITHDRAW='{
  "start_withdraw": {}
}'
$BINARY tx wasm execute "$CONTRACT_ADDRESS" "$START_WITHDRAW" --from test-user $TXFLAG

READY_TIME=$($BINARY q wasm contract-state smart $CONTRACT_ADDRESS '{"get_withdrawal_ready_time": {}}' --output json)
echo $READY_TIME | jq .

# Print out config variables
printf "\n ------------------------ \n"
printf "Contract Variables \n\n"

echo "CODE_ID=$CONTRACT_CODE"
echo "CONTRACT_ADDRESS=$CONTRACT_ADDRESS"

echo $RES
exit $RES
