import {expect} from 'chai';
import {ContractPromise} from '@polkadot/api-contract';
import wrapperContractFile from './resources/flipper-wrapper.contract.json' assert {type: 'json'};
import flipper_abi from "./resources/flipper_abi.js";
import * as path from "path";
import {fileURLToPath} from 'url';
import * as fs from "fs";
import CommonWasm from "../../src/common/CommonWasm.js";
import CommonEvm from "../../src/common/СommonEvm.js";

describe('WASM to EVM communication', async function () {
    this.timeout(60000);

    let commonWasm;
    let commonEvm;

    before(async () => {
        commonWasm = await new CommonWasm().init();
        commonEvm = new CommonEvm();
    })

    after(async () => {
        await commonEvm.disconnect();
        await commonWasm.disconnect();
    });

    it('should call WASM contract from EVM contract', async () => {
        const __filename = fileURLToPath(import.meta.url);
        const __dirname = path.dirname(__filename);
        const contractPath = path.join(__dirname, '../../../examples/cross-vm-communication/wasm-to-evm/flipper.sol');
        const sourceCode = fs.readFileSync(contractPath, "utf8");

        const {abi, bytecode} = await commonEvm.compile(sourceCode, "Flipper");
        const evmAccount = await commonEvm.getAccount();

        const deployResult = await commonEvm.deployContract(abi, bytecode);
        const flipperContractAddress = deployResult.options.address;
        console.log('flipperContractAddress:', flipperContractAddress);

        let flipperValueBefore = await evmContractCall(flipper_abi, flipperContractAddress, commonEvm.getWeb3(), evmAccount);
        console.log('flipperValueBefore: ', flipperValueBefore);

        //assertion before method call:
        expect(flipperValueBefore).to.be.true;

        const wrapperContract = await commonWasm.deployContract(wrapperContractFile);
        const wrapperContractAddress = wrapperContract.address.toString();
        const wrapperContractPromise = new ContractPromise(commonWasm.getApi(), wrapperContractFile, wrapperContractAddress);

        console.log('contractAddress', wrapperContractAddress);

        const storageDepositLimit = null;
        const gasLimit = await commonWasm.getGasLimit();

        const tx = await wrapperContractPromise.tx
            .flip({storageDepositLimit, gasLimit}, flipperContractAddress);
        await commonWasm.signAndSend(tx, commonWasm.getAccount());

        let flipperValueAfter = await evmContractCall(flipper_abi, flipperContractAddress, commonEvm.getWeb3(), evmAccount);
        console.log('flipperValueAfter: ', flipperValueAfter);

        //value must be changed after method call:
        expect(flipperValueAfter).to.be.false;
    });

    async function evmContractCall(abi, address, web3, evmAccount) {
        const contract = new web3.eth.Contract(abi, address);

        return await contract.methods.data()
            .call({from: evmAccount})
            .then(result => {
                console.log('call result:', result);
                return result;
            })
            .catch(err => {
                console.log('call err:', err);
                return err;
            });
    }
});
