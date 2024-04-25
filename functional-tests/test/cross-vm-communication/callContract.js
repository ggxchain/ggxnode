//Code for bug: https://github.com/ggxchain/planning/issues/120

import ctrtJson from "./flipper.json" assert { type: "json"};
import Web3 from "web3";
//import 'jest';
/* Setup Environment
- Install Ubuntu dependencies: 
sudo apt-get -y install binaryen 
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup component add rust-src
rustup component add rust-src --toolchain nightly-unknown-linux-gnu
rustup target add wasm32-unknown-unknown

- Setup EVM Blockchain Account:
GGX Chain Account Private Key: 0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391

Anvil Acccount Private Key:
0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

- Deploy Flipper smart contract(test/cross-vm-communication/Flipper.sol) via Remix or Foundry with GGXChain details:
Network name: 'GGX'
Chain ID: 8866
Currency symbol: GGX
RPC URL = "ws://127.0.0.1:9944" for GGX Chain or  'http://127.0.0.1:8545' for Anvil
"https://testnet.node.sydney.ggxchain.io/" is not working currently

- Install Test Environment:
$ `bun install`
Copy and paste the deployed smart contract address below. 

- Run the GGX Chain:
`cd node && cargo run --release --features=brooklyn --no-default-features -- --dev --tmp`

- Run this test file:
$ `bun run callEvmContract`

See reference at examples/cross-vm-communication/README.md 
*/
const lg = console.log;
let balc;//: bigint;

describe('Call EVM Smart Contracts', function () {
    lg('Here 101');
    this.timeout(30000);

    it('should call WASM contract from EVM contract', async () => {
        //const deployerPK = '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'//anvil 0xf39F
				const deployerPK = '0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391';//ggxnode 0xaaa

        //const nodeUrl = 'http://127.0.0.1:8545';
        const nodeUrl = 'ws://127.0.0.1:9944';
        //const nodeUrl = 'https://testnet.node.sydney.ggxchain.io';

        const web3 = new Web3(nodeUrl);
        const blockNumber = await web3.eth.getBlockNumber();
        const chainId = await web3.eth.getChainId();
        let gasPrice = await web3.eth.getGasPrice();
        lg('blockNumber', blockNumber, ', chainId', chainId, ', gasPrice:', gasPrice)
        const account = web3.eth.accounts.wallet.add(deployerPK);
        const deployer = account[0].address;
        lg('deployer', deployer);
        balc = await web3.eth.getBalance(deployer);
        let nonce = await web3.eth.getTransactionCount(deployer);
        lg('balc:', balc, ', nonce:', nonce);

        const fromWei = (amount) => {
            return web3.utils.fromWei(amount, 'ether')
        }
        const toWei = (amount) => {
            return web3.utils.toWei(amount, 'ether')
        }
        const ctrtAddr = '0xfB13f1A798a1aa6D8359fE4bEAAeF1FD04a8dCD4';
        lg('ctrtAddr:', ctrtAddr);
				
				const abi = ctrtJson.abi;

        const ctrt = new web3.eth.Contract(abi, ctrtAddr );
        let data = await ctrt.methods.data().call().catch(err => {
            lg('error @data():', err);
            return err;
        });
        lg('data:', data);

				const gasEstimate = await ctrt.methods
      .flip().estimateGas({ from: deployer });
				lg('gasEstimate:', gasEstimate);

				const encode = ctrt.methods.flip().encodeABI();
				lg('before txn...')
				const tx = await web3.eth.sendTransaction({
					from: deployer,
					to: ctrtAddr,
					gas: gasEstimate,
					data: encode,
				}).catch(err => {
					lg('error @flip():', err);
					return err;
				});//value: '0x0' or feeInWei.toString(),
				lg('txn:', tx);
	
				lg('Tx hash:',tx.transactionHash);
				
				data = await ctrt.methods.data().call();
        lg('data:', data);
        process.exit(0);
    });
})
