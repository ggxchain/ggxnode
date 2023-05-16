const { Keyring } = require('@polkadot/keyring');
const { CodePromise, ContractPromise } = require('@polkadot/api-contract');
const { mnemonicGenerate } = require('@polkadot/util-crypto');
const contract_file = require('./data/erc20.contract.json');

module.exports = { transferFunds, deployInkErc20, transferErc20Funds, createTestAccount };

/// Transfer funds from the precreated account in (@createTestAccount) to another.
async function transferFunds(userContext, events, done) {
    const receiver = userContext.vars.receiver;
    const sender = userContext.vars.sender;
    extrinstic = userContext.api.tx.balances.transfer(receiver, userContext.funcs.$randomNumber(1));
    try {
        await extrinstic.signAndSend(sender, { nonce: userContext.vars.$loopCount });
    }
    catch (e) { } // The error can happen if transaction pool is full, so we just ignore it and keep going
    return done();
}

function createGasLimit(api) {
    return api.registry.createType('WeightV2', {
        refTime: BigInt(1 * 1e10), // we have 18 decimals, so 1e10 is 0.00000001
        proofSize: 1024 * 1024 * 1, // pretty big proof for contract deployment
    });
}


/// Deploy ERC20 contract and save its address to the context. Uses the precreated account from (@createTestAccount).
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
    try {
        await tx.signAndSend(sender, { nonce: userContext.vars.$loopCount }, ({ contract, status }) => {
            if (status.isFinalized) {
                userContext.vars.inkErc20Address = contract.address.toString();
                done();
            }
        })
    }
    catch (e) { } // The error can happen if transaction pool is full, so we just ignore it and keep going
}


/// Transfer funds from the precreated account in (@createTestAccount) to another using ERC20 contract that was deployed in (@deployInkErc20).
async function transferErc20Funds(userContext, events, done) {
    const sender = userContext.vars.sender;
    const receiver = userContext.vars.receiver;
    const address = userContext.vars.inkErc20Address;

    // Contracts usage
    const contract = new ContractPromise(userContext.api, contract_file, address);
    const gasLimit = createGasLimit(userContext.api);
    const storageDepositLimit = null;
    // +1 because we used the nonce for the contract deployment
    try {
        await contract.tx.transfer({ gasLimit, storageDepositLimit }, receiver, 100).signAndSend(sender, { nonce: userContext.vars.$loopCount + 1 });
    }
    catch (e) { } // The error can happen if transaction pool is full, so we just ignore it and keep going
    return done();
}

/// Creates a spam account and whitelist it. The account is used for stress testing.
async function createTestAccount(userContext, events, done) {
    const keyring = new Keyring({ type: 'sr25519' });
    mnemonic = mnemonicGenerate(12);
    const alice = keyring.addFromUri('//Alice');
    userContext.vars.sender = keyring.addFromMnemonic(mnemonic);
    userContext.vars.receiver = keyring.addFromUri('//Bob').address;
    userContext.vars.alice = alice.address;

    // Might fail because of nonce issue (due to parallelism).
    try {
        const unsub = await userContext.api.tx.balances.transfer(userContext.vars.sender.address, BigInt(1000 * 1e18)).signAndSend(alice, { nonce: -1 }, (result) => {
            if (result.status.isFinalized) {
                // Alice supposed to transfer some funds to it
                unsub();
            }
        });
        // Alice supposed to whitelist the account
        await userContext.api.tx.sudo.sudo(userContext.api.tx.accountFilter.addAccount(userContext.vars.sender.address)).signAndSend(alice, { nonce: -1 }, (result) => {
            if (result.status.isFinalized) {
                done();
            }
        });

    } catch (e) {
        return null;
    }
}
