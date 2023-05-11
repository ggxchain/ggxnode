const { Keyring } = require('@polkadot/keyring');
const { CodePromise, ContractPromise } = require('@polkadot/api-contract');
const { mnemonicGenerate } = require('@polkadot/util-crypto');
const contract_file = require('./data/erc20.contract.json');

module.exports = { transferFunds, deployInkErc20, transferErc20Funds, createTestAccount, returnFunds };


async function transferFunds(userContext, events, done) {
    const receiver = userContext.vars.receiver;
    const sender = userContext.vars.sender;
    extrinstic = userContext.api.tx.balances.transfer(receiver, userContext.funcs.$randomNumber(1));
    extrinstic.signAndSend(sender, { nonce: userContext.vars.$loopCount });

    return done();
}

function createGasLimit(api) {
    return api.registry.createType('WeightV2', {
        refTime: 3000n * 1000000n,
        proofSize: 512 * 1024,
    });
}

async function deployInkErc20(userContext, events, done) {
    const sender = userContext.vars.sender;


    // Contracts preparation
    const code = new CodePromise(userContext.api, contract_file);
    // We don't care about the storage limit, so we set it to null.
    const storageDepositLimit = null;
    const gasLimit = createGasLimit(userContext.api);
    const initBalance = 10000000000;
    const tx = code.tx.new({
        gasLimit,
        storageDepositLimit
    }, initBalance);

    let address;
    const unsub = await tx.signAndSend(sender, { nonce: userContext.vars.$loopCount }, ({ contract, status }) => {
        if (status.isInBlock || status.isFinalized) {
            address = contract.address.toString();
            unsub();
        }
    })
    userContext.vars.inkErc20Address = address;
    return done();
}

async function transferErc20Funds(userContext, events, done) {
    const sender = userContext.vars.sender;
    const receiver = userContext.vars.receiver;
    const address = userContext.vars.inkErc20Address;


    // Contracts usage
    const contract = new ContractPromise(userContext.api, contract_file, address);
    const gasLimit = createGasLimit(userContext.api);
    const storageDepositLimit = null;
    await contract.tx.transfer({ gasLimit, storageDepositLimit }, receiver, 100).signAndSend(sender, { nonce: userContext.vars.$loopCount });
    return done();
}

async function createTestAccount(userContext, events, done) {
    const keyring = new Keyring({ type: 'sr25519' });
    mnemonic = mnemonicGenerate(12);
    const alice = keyring.addFromUri('//Alice');
    userContext.vars.sender = keyring.addFromMnemonic(mnemonic);
    userContext.vars.receiver = keyring.addFromUri('//Bob').address;
    userContext.vars.alice = alice.address;

    // Alice supposed to whitelist the sender and transfer some funds to it
    await userContext.api.tx.balances.transfer(userContext.vars.sender.address, BigInt(1000 * 1e18)).signAndSend(alice, { nonce: -1 });
    await userContext.api.tx.sudo.sudo(userContext.api.tx.accountFilter.addAccount(userContext.vars.sender.address)).signAndSend(alice, { nonce: -1 });

    return done();
}

async function returnFunds(userContext, events, done) {
    // Return balance to Alice
    const system_account = await userContext.api.query.system.account(userContext.vars.sender.address);
    const balance = system_account.data.free.toBigInt();

    const alice = userContext.vars.alice;
    /// -10000 to cover the tx fee
    await userContext.api.tx.balances.transfer(alice, balance - 10000n).signAndSend(userContext.vars.sender, { nonce: -1 });

    return done();
}