import {ContractPromise} from '@polkadot/api-contract';
import contract_file from './assets/flipper.contract.json' assert {type: 'json'};
import xvm_abi from "./assets/xvm_abi.js";
import CommonWasm from "../../src/common/CommonWasm.js";
import CommonEvm from "../../src/common/СommonEvm.js";
import {expect} from "chai";

describe('EVM to WASM communication', async function () {
    this.timeout(60000);

    let commonWasm;
    let commonEvm;

    before( async () => {
        commonWasm = await new CommonWasm().init();
        commonEvm = new CommonEvm();
    })

    after(async function ()  {
        await commonEvm.disconnect();
        await commonWasm.disconnect();
    });

    it('should call WASM contract from EVM contract', async function ()  {
        const initBalance = 1;
        const flipperContract = await commonWasm.deployContract(contract_file, initBalance);
        const flipperContractAddress = flipperContract.address.toString();
        const flipperContractPublicKey = await commonWasm.getContractPublicKey(flipperContractAddress);

        const contractPromise = new ContractPromise(await commonWasm.getApi(), contract_file, flipperContractAddress);
        console.log('contractAddress', flipperContractAddress);

        let flipperValueBefore = await getWasmFlipperValue(contractPromise);
        expect(flipperValueBefore).to.be.true;

        //call Flipper.flip() from EVM:
        const evmAccount = await commonEvm.getAccount();
        console.log('evmAccount', evmAccount);

        const xvmContractAddress = '0x0000000000000000000000000000000000005005';
        const xvmContract = commonEvm.getContract(xvm_abi, xvmContractAddress);

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
                console.error('Error calling contractPromise method:', error);
            });

        //verify that value is changed:
        const flipperValueAfter = await getWasmFlipperValue(contractPromise);
        expect(flipperValueAfter).to.be.false;
    });

    async function getWasmFlipperValue(contractPromise) {
        const storageDepositLimit = null;
        const gasLimit = await commonWasm.getGasLimit();

        let queryResult = await contractPromise.query.get(
            commonWasm.getAccount().address, {
                gasLimit,
                storageDepositLimit,
            }
        );

        let dataValue = queryResult.result.toHuman().Ok.data;
        console.log('flipper value: ', dataValue);

        return commonWasm.hexToBoolean(dataValue);
    }
});
