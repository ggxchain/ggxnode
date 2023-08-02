import HDWalletProvider from "@truffle/hdwallet-provider";
import Web3 from "web3";
import solc from "solc";

class CommonEvm {
    nodeUrl = 'https://testnet.node.sydney.ggxchain.io';
    mnemonic = 'movie avoid rack lesson rival rice you average caution eternal distance wood';

    constructor() {
        this.provider = new HDWalletProvider({
            mnemonic: this.mnemonic,
            providerOrUrl: this.nodeUrl
        });

        this.web3 = new Web3(this.provider);
    }

    getWeb3() {
        return this.web3;
    }

    getContract(abi, address) {
        return new this.web3.eth.Contract(abi, address);
    }

    async getAccount() {
        return new Promise(async (resolve, reject) => {
            try {
                const account = (await this.getWeb3().eth.getAccounts())[0];
                resolve(account);
            } catch (e) {
                reject(e);
            }
        })
    }

    async compile(sourceCode, contractName) {
        return new Promise((resolve, reject) => {
            // Create the Solidity Compiler Standard Input and Output JSON
            const input = {
                language: "Solidity",
                sources: {main: {content: sourceCode}},
                settings: {outputSelection: {"*": {"*": ["abi", "evm.bytecode"]}}},
            };

            // Parse the compiler output to retrieve the ABI and Bytecode
            const output = solc.compile(JSON.stringify(input));
            const artifact = JSON.parse(output).contracts.main[contractName];

            resolve({
                abi: artifact.abi,
                bytecode: artifact.evm.bytecode.object,
            });
        })
    }

    async deployContract(abi, bytecode) {
        return new Promise(async (resolve, reject) => {
            try {
                const accounts = await this.getWeb3().eth.getAccounts();

                console.log('Attempting to deploy from account', accounts[0]);
                const transactionParameters = {
                    from: accounts[0],
                    gas: '3000000',
                };

                const contract = this.getContract(abi);
                const result = await contract.deploy({
                    data: '0x' + bytecode,
                    arguments: [true]
                })
                    .send(transactionParameters);

                console.log('Contract deployed to', result.options.address);

                resolve(result);
            } catch (e) {
                reject(e);
            }
        })
    };

    disconnect() {
        this.provider.engine.stop();
    }
}

export default CommonEvm;
