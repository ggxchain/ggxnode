# Functional tests

This repository contains autotests for GGX Chain which were written with the help of Polkadot-JS API library.

## Before running
Test accounts (`qHWna6gv3F9m85wi9t5S9b8LszFGwffDTXx2uvG1Z814ktoa7` and `0x107f3cCC0Ac5aC2950A2A3860029b2677F3B79CC`) must have some balance to be able to run the tests.  
Mnemonic phrases for these accounts are placed in `src/common/CommonWasm` and `src/common/CommonEvm`.

For now, the tests are configured to run on Sydney Testnet. To run on a different environment you need to change `nodeUrl` variable in both `src/common/CommonWasm` and `src/common/CommonEvm`.  

## How to run

Open in terminal `ggxnode/functional-tests` folder and run the command:

```bash
npm test
```

It will run all the available tests. To run a specific test, use `mocha` command. For example:
```bash
mocha test/staking/staking.test.js
```