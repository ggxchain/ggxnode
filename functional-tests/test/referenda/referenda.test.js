import CommonWasm from "../../src/common/CommonWasm.js";
import {expect} from "chai";

describe('Referenda', async function () {
    this.timeout(60000);

    let commonWasm;

    before( async () => {
        commonWasm = await new CommonWasm().init();
    })

    after(async function ()  {
        await commonWasm.disconnect();
    });

    it('should able to submit proposal', async function ()  {
        const preimageHash = '0x30d961e7469425942c5c06a020b63c68d25ec5c36caf9bed88c6485da238f848';
        const proposalOrigin = 'system';
        const proposal = {'Legacy': preimageHash};
        const enactmentMoment = 'After';

        const referendumCountBefore = await commonWasm.getApi().query.referenda.referendumCount();
        console.log('Referendum count before:', referendumCountBefore.toString());

        const proposalExtrinsic = commonWasm.getApi().tx.referenda.submit(
            proposalOrigin, proposal, enactmentMoment);
        await commonWasm.signAndSend(proposalExtrinsic, commonWasm.getAccount());

        const referendumCountAfter = await commonWasm.getApi().query.referenda.referendumCount();
        console.log('Referendum count after:', referendumCountAfter.toString());

        const referendumInfo = await commonWasm.getApi().query.referenda.referendumInfoFor(referendumCountAfter - 1);
        console.log('referendumInfo:', referendumInfo.toJSON());

        const referendumPreimageHash = referendumInfo.toJSON().ongoing.proposal.legacy.hash;
        expect(referendumPreimageHash).to.be.equal(preimageHash);
        expect(referendumCountAfter - referendumCountBefore).to.be.equal(1);
    });

    it('should able to place decision deposit', async function ()  {
        const referendumIndex = await commonWasm.getApi().query.referenda.referendumCount() - 1;

        const proposalExtrinsic = commonWasm.getApi().tx.referenda.placeDecisionDeposit(referendumIndex);
        await commonWasm.signAndSend(proposalExtrinsic, commonWasm.getAccount());

        const referendumInfo = await commonWasm.getApi().query.referenda.referendumInfoFor(referendumIndex);
        console.log('referendumInfo:', referendumInfo.toJSON());

        const referendumDepositAccount = referendumInfo.toJSON().ongoing.decisionDeposit.who;
        const referendumDepositAmount = commonWasm.hexToDecimal(referendumInfo.toJSON().ongoing.decisionDeposit.amount);

        expect(referendumDepositAccount).to.be.equal(commonWasm.getAccount().address);
        expect(referendumDepositAmount).to.be.equal(5e20);
    });
});
