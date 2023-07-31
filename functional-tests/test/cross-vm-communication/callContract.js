//Code for bug: https://github.com/ggxchain/planning/issues/120

import flipper_abi from "./flipper_abi.js";
import HDWalletProvider from "@truffle/hdwallet-provider";
import Web3 from "web3";

async function evmContractCall(abi, address, web3, evmAccount) {
    const contract = new web3.eth.Contract(abi, address);

    return await contract.methods.data().call()
        .then(result => {
            console.log('call result', result);
            return result;
        })
        .catch(err => {
            console.log('call err', err);
            return err;
        });
}

describe('WASM to EVM communication', function () {
    this.timeout(30000);

    it('should call WASM contract from EVM contract', async () => {
        const nodeUrl = 'https://testnet.node.sydney.ggxchain.io';
        //commented code is used for local Ganache blockchain
        // const nodeUrl = 'HTTP://127.0.0.1:7545';
        let provider = new HDWalletProvider({
            mnemonic: 'movie avoid rack lesson rival rice you average caution eternal distance wood',
            // mnemonic: 'hole casual this royal erase raise address badge meadow excite start amazing',
            providerOrUrl: nodeUrl
        });

        let web3 = new Web3(provider);
        const evmAccount = (await web3.eth.getAccounts())[0];
        console.log('evmAccount', evmAccount)
        await evmContractCall(flipper_abi, '0x6574961660812b8Dfd012743991174B1B7dEA3d3', web3, evmAccount); //sydney
        // await evmContractCall(flipper_abi, '0xe8dc324b2ad33B425b9fd7cC737dECC6549fCC56', web3, evmAccount);
    });
})
