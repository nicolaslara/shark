#!/bin/bash
set -eux
BINARY="../osmosis/build/osmosisd --home ../osmosis/home/"
DEFAULT_DEV_ADDRESS=$($BINARY keys show validator -a --keyring-backend test)
DENOM='uosmo'
CHAIN_ID='localnet-0'
RPC='http://localhost:26657/'
TXFLAG="--gas-prices 0.1$DENOM --gas auto --gas-adjustment 1.3 -y -b block --chain-id $CHAIN_ID --node $RPC --keyring-backend test"
BLOCK_GAS_LIMIT=${GAS_LIMIT:-100000000} # should mirror mainnet

echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

# compile
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --volume $(dirname "$(pwd)")/osmosis-bindings/:/osmosis-bindings \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6


# you ideally want to run locally, get a user and then
# pass that addr in here
echo "Address to deploy contracts: $DEFAULT_DEV_ADDRESS"
echo "TX Flags: $TXFLAG"

# upload  wasm
CONTRACT_CODE=$($BINARY tx wasm store "./artifacts/shark.wasm" --from $DEFAULT_DEV_ADDRESS $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')
echo "Stored: $CONTRACT_CODE"

# ToDo: Use the variable above
INIT='{"admin":null, "funds_denom": "uosmo", "collateral_denom": "gamm/pool/1"}'
echo "$INIT" | jq .
$BINARY tx wasm instantiate $CONTRACT_CODE "$INIT" --from $DEFAULT_DEV_ADDRESS --label "shark" $TXFLAG --no-admin
RES=$?
# get contract addr
CONTRACT_ADDRESS=$($BINARY query wasm list-contract-by-code $CONTRACT_CODE --output json | jq -r '.contracts[-1]')

echo $CONTRACT_ADDRESS

printf "\n ------------------------ \n"
printf "Config Variables \n\n"

echo "NEXT_PUBLIC_SHARK_CODE_ID=$CONTRACT_CODE"
echo "NEXT_PUBLIC_SHARK_ADDRESS=$CONTRACT_ADDRESS"

export contract=$CONTRACT_ADDRESS
echo "contract address exported to \$contract\n"

echo $RES
exit $RES
