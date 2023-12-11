import HDWalletProvider from "@truffle/hdwallet-provider";
import Web3 from "web3";
import solc from 'solc';

class CommonEvm {
    nodeUrl = 'https://sydney-archive.dev.ggxchain.io:9944';
    mnemonic = 'movie avoid rack lesson rival rice you average caution eternal distance wood';
    compilerVersion = 'v0.8.19+commit.7dd6d404';

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
                // Solidity Compiler Standard Input and Output JSON
                const input = {
                    language: "Solidity",
                    sources: {main: {content: sourceCode}},
                    settings: {outputSelection: {"*": {"*": ["abi", "evm.bytecode"]}}},
                };

                solc.loadRemoteVersion(this.compilerVersion, async (err, solcSnapshot) => {
                    if (err) {
                        console.error(err);
                    } else {
                        const output = solcSnapshot.compile(JSON.stringify(input));
                        const artifact = JSON.parse(output).contracts.main[contractName];
                        resolve({
                            abi: artifact.abi,
                            bytecode: artifact.evm.bytecode.object,
                        });
                    }
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
                const account = await this.getAccount();

                console.log('Attempting to deploy contract from account', account);

                const transactionParameters = await this.getTransactionParameters();

                const contract = this.getContract(abi);

                contract.deploy({
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
    }

    async getTransactionParameters() {
        return {
            from: await this.getAccount(),
            gasPrice: 3500000000,
            gas: 900000,
        };
    }

    disconnect() {
        this.provider.engine.stop();
    }
}

export default CommonEvm;
