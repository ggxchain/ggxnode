import CommonWasm from "../../src/common/CommonWasm.js";
import {expect} from "chai";

describe('Referenda', async function () {
    this.timeout(60000);

    const SYSTEM_REMARK_PREIMAGE_HASH = '0xbbf5004add4e25e7a065b96aca30e2ee7cd5e15c3d91203354b685fbfcc8d09c';
    let commonWasm;

    before( async () => {
        commonWasm = await new CommonWasm().init();

        await createPreimageForTests();
    })

    after(async function ()  {
        await commonWasm.disconnect();
    });

    it('should able to submit proposal', async function ()  {
        const proposalOrigin = 'system';
        const proposal = {'Legacy': SYSTEM_REMARK_PREIMAGE_HASH};
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
        expect(referendumPreimageHash).to.be.equal(SYSTEM_REMARK_PREIMAGE_HASH);
        expect(referendumCountAfter - referendumCountBefore).to.be.equal(1);
    });

    it('should able to place decision deposit', async function ()  {
        const referendumIndex = await getCurrentReferendumIndex();
        console.log('referendumIndex:', referendumIndex);

        const proposalExtrinsic = commonWasm.getApi().tx.referenda.placeDecisionDeposit(referendumIndex);
        await commonWasm.signAndSend(proposalExtrinsic, commonWasm.getAccount());

        const referendumInfo = await commonWasm.getApi().query.referenda.referendumInfoFor(referendumIndex);
        console.log('referendumInfo:', referendumInfo.toJSON());

        const referendumDepositAccount = referendumInfo.toJSON().ongoing.decisionDeposit.who;
        const referendumDepositAmount = commonWasm.hexToDecimal(referendumInfo.toJSON().ongoing.decisionDeposit.amount);

        expect(referendumDepositAccount).to.be.equal(commonWasm.getAccount().address);
        expect(referendumDepositAmount).to.be.equal(5e20);
    });

    it('should be able to set metadata', async () => {
        const PREIMAGE_HASH = '0x30d961e7469425942c5c06a020b63c68d25ec5c36caf9bed88c6485da238f848';
        const referendumIndex = await getCurrentReferendumIndex();
        console.log('referendumIndex:', referendumIndex);

        const metadataExtrinsic = commonWasm.getApi().tx.referenda.setMetadata(referendumIndex, PREIMAGE_HASH);
        await commonWasm.signAndSend(metadataExtrinsic, commonWasm.getAccount());

        const metadataOf = (await commonWasm.getApi().query.referenda.metadataOf(referendumIndex)).toJSON();
        console.log('metadataOf:', metadataOf);

        expect(metadataOf).to.be.equal(PREIMAGE_HASH);
    });

    it('should be able to get deciding count', async () => {
        const referendumIndex = await getCurrentReferendumIndex();
        console.log('referendumIndex:', referendumIndex);

        const decidingCount = (await commonWasm.getApi().query.referenda.decidingCount(referendumIndex)).toJSON();
        console.log('decidingCount:', decidingCount);

        expect(decidingCount).to.be.at.least(1);
    });

    it('should be able to get referenda metadata', async () => {
        const referendumIndex = await getCurrentReferendumIndex();
        console.log('referendumIndex:', referendumIndex);

        const metadataOf = (await commonWasm.getApi().query.referenda.metadataOf(referendumIndex)).toJSON();
        console.log('metadataOf:', metadataOf);

        expect(metadataOf).to.match(/0x[0-9a-fA-F]+/);
    });

    it('should be able to get referenda pallet version', async () => {
        const palletVersion = (await commonWasm.getApi().query.referenda.palletVersion()).toJSON();
        console.log('palletVersion:', palletVersion);

        expect(palletVersion).to.match(/^\d+$/);
    });

    it('should be able to get referendum count', async () => {
        const referendumCount = (await commonWasm.getApi().query.referenda.referendumCount()).toJSON();
        console.log('referendumCount:', referendumCount);

        expect(referendumCount).to.be.at.least(1);
    });

    it('should be able to get referendum info', async () => {
        const referendumIndex = await getCurrentReferendumIndex();
        console.log('referendumIndex:', referendumIndex);

        const referendumInfo = await commonWasm.getApi().query.referenda.referendumInfoFor(referendumIndex);
        console.log('referendumInfo:', referendumInfo.toJSON());

        const proposalHash = referendumInfo.toJSON().ongoing.proposal.legacy.hash;
        const enactment = referendumInfo.toJSON().ongoing.enactment.after;
        const submitted = referendumInfo.toJSON().ongoing.submitted;
        const submissionDepositAccount = referendumInfo.toJSON().ongoing.submissionDeposit.who;
        const submissionDepositAmount = commonWasm.hexToDecimal(referendumInfo.toJSON().ongoing.submissionDeposit.amount);

        expect(proposalHash).to.match(/0x[0-9a-fA-F]+/);
        expect(enactment).to.match(/^\d+$/);
        expect(submitted).to.match(/^\d+$/);
        expect(submissionDepositAccount).to.be.equal(commonWasm.getAccount().address);
        expect(submissionDepositAmount).to.be.equal(1_000_000_000_000_000_000);
    });

    it('should be able to get referendum track queue', async () => {
        const referendumIndex = await getCurrentReferendumIndex();
        console.log('referendumIndex:', referendumIndex);

        const trackQueue = (await commonWasm.getApi().query.referenda.trackQueue(referendumIndex)).toJSON();
        console.log('trackQueue:', trackQueue);

        expect(trackQueue).to.be.an('array');
    });

    async function createPreimageForTests() {
        const SYSTEM_REMARK_BYTES = '0x000003';
        const preimageStatus = await commonWasm.getApi().query.preimage.statusFor(SYSTEM_REMARK_PREIMAGE_HASH);

        if (preimageStatus.toJSON() == null) {
            const extrinsic = commonWasm.getApi().tx.preimage.notePreimage(SYSTEM_REMARK_BYTES);
            await commonWasm.signAndSend(extrinsic, commonWasm.getAccount());
        }
    }

    async function getCurrentReferendumIndex() {
        return await commonWasm.getApi().query.referenda.referendumCount() - 1;
    }
});
