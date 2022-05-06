# Shark

Simple borrowing/lending contract built with CosmWasm on top of Osmosis.

## Requirements

### Osmosis

This project assumes you have a local fork of Osmosis that
exposes the lock-tokens endpoint for cosmwasm. This is still in development,
so you'll need a fork with that implementation (see https://github.com/nicolaslara/osmosis/tree/lock-wasm).

Some of the scripts used for development expect that code to live on `../osmosis/`.

### Osmosis Bindings

Similarly, rust/wasm bindings for the osmosis lock-tokens endpoints are expected.
These are currently in development, but a minimal implementation
can be found in https://github.com/nicolaslara/osmosis-bindings/tree/lock-tokens

### Other tools

For development, this project uses a few helper scripts that depend on `gsed` and `yq`.

## Instalation

ToDo: Document how to set this up from scratch

## Scripts

ToDo: Document helper scripts
