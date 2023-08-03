import CommonWasm from "../../src/common/CommonWasm.js";
import {expect} from "chai";

describe('Staking', async function () {
    this.timeout(60000);

    let commonWasm;

    before( async () => {
        commonWasm = await new CommonWasm().init();
    })

    after(async function ()  {
        await commonWasm.disconnect();
    });

    it('should able to perform bond', async function ()  {
        const senderAddress = commonWasm.getAccount().address;
        const amountToBond = 100000000000000000n;

        const stakingLedgerBefore = await commonWasm.getApi().query.staking.ledger(senderAddress);
        console.log('Staking Ledger Before:', stakingLedgerBefore.toJSON());

        const transaction = commonWasm.getApi().tx.staking.bondExtra(amountToBond);
        await commonWasm.signAndSend(transaction, commonWasm.getAccount());

        const stakingLedgerAfter = await commonWasm.getApi().query.staking.ledger(senderAddress);
        console.log('Staking Ledger After:', stakingLedgerAfter.toJSON());

        const difference = BigInt(commonWasm.hexToDecimal(stakingLedgerAfter.toJSON().total)
            - commonWasm.hexToDecimal(stakingLedgerBefore.toJSON().total));

        expect(difference).to.be.equal(amountToBond);
    });

    it('should able to perform unbond', async function ()  {
        const senderAddress = commonWasm.getAccount().address;
        const amountToUnbond = 100000000000000000n;

        const stakingLedgerBefore = await commonWasm.getApi().query.staking.ledger(senderAddress);
        console.log('Staking Ledger Before:', stakingLedgerBefore.toJSON());

        const transaction = commonWasm.getApi().tx.staking.unbond(amountToUnbond);
        await commonWasm.signAndSend(transaction, commonWasm.getAccount());

        const stakingLedgerAfter = await commonWasm.getApi().query.staking.ledger(senderAddress);
        console.log('Staking Ledger After:', stakingLedgerAfter.toJSON());

        const difference = BigInt(commonWasm.hexToDecimal(stakingLedgerBefore.toJSON().active)
            - commonWasm.hexToDecimal(stakingLedgerAfter.toJSON().active));

        expect(difference).to.be.equal(amountToUnbond);
    });
});
