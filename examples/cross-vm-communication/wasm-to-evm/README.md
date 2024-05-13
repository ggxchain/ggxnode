## Steps Of Deploying WASM XVM

#### Install Rust
- Rust v1.78.0 is okay
- Set Rust to the stable version to enable rust-toolchain.toml: `rustup default stable`

#### Download this repository code
- Make sure Wasm folder is excluded from compilation in the root `Cargo.toml`: `"examples/cross-vm-communication/wasm-to-evm",`

- Compile and run it with data persistency: `cargo run --features=brooklyn -- --dev --rpc-external --unsafe-rpc-external --rpc-methods=unsafe -d ../data`

#### Setup EVM Tool
Configure your wallet e.g. Metamask:
- Name: GGX,
- URL: http://127.0.0.1:9944 ,
- ChainID: 888866
- Currency Symbol: GGX

Import the test Ethereum account
Private key: `0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391`, which has the Public key `0xaaafB3972B05630fCceE866eC69CdADd9baC2771`

Wait for the balance to show up after a little while...

#### Deploy Solidity contract
- Use EVM tools like Remix connecting to the local node from above 
- Find the Storage contract located inside `examples/cross-vm-communication/wasm-to-evm/contracts/solidity/Simple.sol`
- Use Solidity compiler `0.8.25` to compile the contract and deploy it
- Call the read function `retrieve()` and save the initial variable value
- in Remix. Go to `File Explorer` > `contracts` > `artifacts` > Flipper.json, copy the `methodIdentifiers`:
```
"methodIdentifiers": {
			"data()": "73d4a13a",
			"flip()": "cde4efa9"
		}
```

#### Install Ink CLI tool `Cargo contract`
install it inside a repository without a rust toolchain file:
```
cargo install --force --locked cargo-contract
cargo contract --version
```
cargo contract version is v4.1.1 works great with Wasm Ink! 4.3.0 and 5.0.0 version

#### Write Xvm Wasm contract
- Update module name according to Cargo.toml: `flipper_xvm`
- Update the module struct name
- Update the Selector names and their hashes from the `methodIdentifiers` above
- Update the function names and their arguments

#### Compile wrapper wasm contract:
`cargo contract build --manifest-path examples/cross-vm-communication/wasm-to-evm/contracts/flipper_xvm/Cargo.toml`
... this should succeed with the following:
  - flipper_xvm.contract (code + metadata)
  - flipper_xvm.wasm (the contract's code)
  - flipper_xvm.json (the contract's metadata)

#### Deploy the XVM Wasm
Use https://ui.use.ink/ to deploy wasm contracts because it shows good error messages
- the constructor argument: `evmAddress` = the deployed Solidity contract address from above
- Click on the tab next to `Storage Deposit Limit`, then enter 999 or 999999 or something big enough 
- Click on `Next` then `instantiate` to deploy it
- After deployment, it may say `Contract Reverted. Decoding Failed`. As long as you have a deployed Xvm Wasm address, ignore the error.
- Copy the deployed Xvm Wasm address displayed on the top

#### Invoke Write Functions via PolkadotJS
Use Polkadot.js because it can invoke write functions
- Go to https://polkadot.js.org/apps/#/contracts
- Choose `add existing contract`
- Contract Address = the Xvm Wasm address from above
- Contract Name = `flipper`
- ABI = the `flipper_xvm.contract` file from above.

Then under `contracts`, click on the little triangle to expand the contract methods. 
Click on `exec` next to the method you want. Enter a value, and execute the transaction.

Wait 20 seconds for the blockchain node to process the transaction.

#### Confirm State Change
- Go back to the Ethereum tool and call the read function again. 
- Confirm the onchain variable value has been changed to the new value

#### Referance
https://medium.com/astar-network/cross-virtual-machine-creating-a-portal-to-the-future-of-smart-contracts-a96c6d2f79b8

https://theastarbulletin.news/how-to-implement-a-contract-using-xvm-1c94d2072c30

https://substrate.stackexchange.com/questions/11435/xvm-ink-wasm-to-evm-contract-reverted-decoding-failed

https://docs.astar.network/docs/learn/interoperability/xvm/#interfaces