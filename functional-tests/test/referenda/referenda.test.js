import CommonWasm from "../../src/common/CommonWasm.js";
import {expect} from "chai";
import { blake2AsU8a } from '@polkadot/util-crypto';

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
        const preimageHash = '0xe8a006a5cbdfedb257da7778e16e34bfac83bc4899485e0a81d2beb037a43f1c';
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
});
