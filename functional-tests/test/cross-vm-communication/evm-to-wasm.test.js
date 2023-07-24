import {expect} from 'chai';
import {ApiPromise, Keyring, WsProvider} from '@polkadot/api';
import {CodePromise, ContractPromise} from '@polkadot/api-contract';
import contract_file from './flipper.contract.json' assert {type: 'json'};
import {decodeAddress} from '@polkadot/util-crypto';
import Web3 from 'web3';
import HDWalletProvider from '@truffle/hdwallet-provider';
import xvm_abi from "./xvm_abi.js";
import { cryptoWaitReady } from '@polkadot/util-crypto';

describe('EVM to WASM communication', function () {
    this.timeout(30000);

    const connect = async () => {
        const provider = new WsProvider('wss://testnet.node.brooklyn.ggxchain.io');
        return await ApiPromise.create({provider});
    };

    async function deployContract(api, sender) {
        return new Promise(async (resolve, reject) => {
            try {
                const code = new CodePromise(api, contract_file);
                const storageDepositLimit = null;
                const gasLimit = createGasLimit(api);
                const initBalance = 1;

                const tx = code.tx.new({gasLimit, storageDepositLimit}, initBalance);

                const contract = await signAndSend(tx, sender);
                resolve(contract);
            } catch (e) {
                reject(e);
            }
        })
    }

    async function signAndSend(tx, sender) {
        return new Promise(async (resolvePromise, reject) => {
            await tx.signAndSend(sender, {nonce: -1}, (result) => {
                console.log(`Current status is ${result.status}`);
                if (result.status.isFinalized) {
                    resolvePromise(result.contract);
                }
            });
        })
    }

    function createGasLimit(api) {
        return api.registry.createType('WeightV2', {
            refTime: BigInt(1e10),
            proofSize: 1024 * 1024,
        });
    }

    async function getContractPublicKey(api, contractAddress) {
        return new Promise((resolve, reject) => {
            try {
                const accountId = decodeAddress(contractAddress);
                resolve(u8aToHex(accountId));
            } catch (error) {
                reject('Error getting account details: ' + error);
            }
        })
    }

    function u8aToHex(u8a) {
        return '0x00' + Array.from(u8a, (byte) => ('00' + byte.toString(16)).slice(-2)).join('');
    }

    function hexToBoolean(hexValue) {
        return !!parseInt(hexValue);
    }


    it('should call WASM contract from EVM contract', async () => {
        await cryptoWaitReady();
        const keyring = new Keyring({type: 'sr25519'});
        const sender = keyring.addFromMnemonic('judge decrease owner toddler face album actor diary require junk crater grape');

        const nodeUrl = 'https://testnet.node.brooklyn.ggxchain.io?chainId=8866';
        let provider = new HDWalletProvider({
            mnemonic: 'movie avoid rack lesson rival rice you average caution eternal distance wood',
            providerOrUrl: nodeUrl
        });

        let web3 = new Web3(provider);

        const api = await connect();

        const flipperContract = await deployContract(api, sender);
        const flipperContractAddress = flipperContract.address.toString();
        const flipperContractPublicKey = await getContractPublicKey(api, flipperContractAddress);

        const contract = new ContractPromise(api, contract_file, flipperContractAddress);

        console.log('contractAddress', flipperContractAddress);

        const storageDepositLimit = null;
        const gasLimit = createGasLimit(api);
        let queryResult = await contract.query.get(
            sender.address,
            {
                gasLimit,
                storageDepositLimit,
            }
        );

        console.log('result before: ', queryResult.result.toHuman().Ok.data);

        let dataValue = queryResult.result.toHuman().Ok.data;
        let dataValueBoolean = hexToBoolean(dataValue);
        expect(dataValueBoolean).to.be.true;

        const evmAccount = (await web3.eth.getAccounts())[0];

        const xvmContractAddress = '0x0000000000000000000000000000000000005005';
        const xvmContract = new web3.eth.Contract(xvm_abi, xvmContractAddress);
        const transactionParameters = {
            from: evmAccount,
            gasPrice: '20000000000',
            gas: '3000000',
        };

        await xvmContract.methods.xvm_call('0x1f0700e87648170284d71700', flipperContractPublicKey, '0xDEADBEEF')
            .send(transactionParameters)
            .on('transactionHash', (hash) => {
                console.log('Transaction Hash:', hash);
            })
            .on('receipt', (receipt) => {
                console.log('Transaction Receipt:', receipt);
            })
            .on('error', (error) => {
                console.error('Error calling contract method:', error);
            });

        const blockNumber = await web3.eth.getBlockNumber();
        console.log('latest blockNumber:', blockNumber);
        console.log('address:', (await web3.eth.getAccounts())[0]);

        queryResult = await contract.query.get(
            sender.address,
            {
                gasLimit,
                storageDepositLimit,
            }
        );

        dataValue = queryResult.result.toHuman().Ok.data;

        console.log('dataValue after: ', dataValue);

        provider.engine.stop();
        await api.disconnect();

        dataValueBoolean = hexToBoolean(dataValue);
        expect(dataValueBoolean).to.be.false;
    });
});
