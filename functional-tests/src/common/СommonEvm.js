import HDWalletProvider from "@truffle/hdwallet-provider";
import Web3 from "web3";
import solc from "solc";

class CommonEvm {
    nodeUrl = 'https://sydney-archive.dev.ggxchain.io:9944';
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
            try {
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
            } catch (e) {
                reject(e);
            }
        })
    }

    async deployContract(abi, bytecode) {
        const web3 = this.getWeb3();

        return new Promise(async (resolve, reject) => {
            try {
                const accounts = await this.getWeb3().eth.getAccounts();

                console.log('Attempting to deploy from account', accounts[0]);
                const transactionParameters = {
                    from: accounts[0],
                    gas: '3000000',
                    gasLimit: '3000000'
                };

                const contract = this.getContract(abi);

                await contract.deploy({
                    data: '0x' + bytecode,
                    arguments: [true]
                }).send(transactionParameters).on('receipt', function (receipt) {
                    console.log('deployContract receipt', receipt);

                    const checksummedContractAddress = web3.utils.toChecksumAddress(receipt.contractAddress);
                    resolve(checksummedContractAddress);
                });
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
