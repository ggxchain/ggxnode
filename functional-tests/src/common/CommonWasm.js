import {ApiPromise, Keyring, WsProvider} from "@polkadot/api";
import {cryptoWaitReady, decodeAddress} from "@polkadot/util-crypto";
import {CodePromise} from "@polkadot/api-contract";

class CommonWasm {
    nodeUrl = 'wss://sydney-archive.dev.ggxchain.io:9944'
    mnemonic = 'judge decrease owner toddler face album actor diary require junk crater grape'

    constructor() {
    }

    async init() {
        return new Promise( async (resolve, reject) => {
            try {
                await cryptoWaitReady();
                this.keyring = new Keyring({ss58Format: 8886, type: 'sr25519'});
                this.account = this.keyring.addFromMnemonic(this.mnemonic);

                const provider = new WsProvider(this.nodeUrl);
                this.api = await ApiPromise.create({provider});
                resolve(this);
            } catch (e) {
                reject(e);
            }
        })
    }

    getApi() {
        return this.api;
    }

    getAccount() {
        return this.account;
    }

    async deployContract(contractFile, ...args) {
        return new Promise(async (resolve, reject) => {
            try {
                const code = new CodePromise(this.getApi(), contractFile);
                const storageDepositLimit = null;
                const gasLimit = await this.getGasLimit(this.getApi());

                const tx = code.tx.new({gasLimit, storageDepositLimit}, ...args);

                const result = await this.signAndSend(tx, this.account);
                resolve(result.contract);
            } catch (e) {
                reject(e);
            }
        })
    }

    async signAndSend(tx, account) {
        return new Promise(async (resolve, reject) => {
            try {
                tx.signAndSend(account, {nonce: -1}, (result) => {
                    console.log(`Current status is ${result.status}`);
                    if (result.status.isFinalized) {
                        resolve(result);
                    }
                })
            } catch (e) {
                reject(e);
            }
        })
    }

    getGasLimit() {
        return new Promise( (resolve, reject) => {
            try {
                const gasLimit = this.getApi().registry.createType('WeightV2', {
                    refTime: BigInt(1e10),
                    proofSize: 1024 * 1024,
                });
                resolve(gasLimit);
            } catch (e) {
                reject(e);
            }
        })
    }

    async getContractPublicKey(contractAddress) {
        return new Promise((resolve, reject) => {
            try {
                const accountId = decodeAddress(contractAddress);
                resolve(this.u8aToHex(accountId));
            } catch (error) {
                reject('Error getting account details: ' + error);
            }
        })
    }

    u8aToHex(u8a) {
        return '0x00' + Array.from(u8a, (byte) => ('00' + byte.toString(16)).slice(-2)).join('');
    }

    hexToBoolean(hexValue) {
        return !!parseInt(hexValue);
    }

    hexToDecimal(hexValue) {
        return parseInt(hexValue, 16);
    }

    disconnect() {
        this.getApi().disconnect();
    }
}

export default CommonWasm;
