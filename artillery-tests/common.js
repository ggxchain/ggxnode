const { Keyring } = require('@polkadot/keyring');
const { CodePromise, ContractPromise } = require('@polkadot/api-contract');
const { mnemonicGenerate } = require('@polkadot/util-crypto');
const contract_file = require('./data/erc20.contract.json');

module.exports = { transferFunds, deployInkErc20, transferErc20Funds, createTestAccount, returnFunds };

function createGasLimit(api) {
    return api.registry.createType('WeightV2', {
        refTime: BigInt(1 * 1e10), // we have 18 decimals, so 1e10 is 0.00000001
        proofSize: 1024 * 1024 * 1, // pretty big proof for contract deployment
    });
}

function signWrapper(done, userContext, callback) {
    function wrapper(result) {
        if (result.isError) {
            userContext.vars.dead = true;
            return done();
        }
        if (result.status.isFinalized) {
            callback(result);
        }
    }
    return wrapper;
}

/// Transfer funds from the precreated account in (@createTestAccount) to another.
async function transferFunds(userContext, events, done) {
    if (userContext.vars.dead) {
        return done();
    }
    const receiver = userContext.vars.receiver;
    const sender = userContext.vars.sender;
    extrinstic = userContext.api.tx.balances.transfer(receiver, userContext.funcs.$randomNumber(1));
    try {
        await extrinstic.signAndSend(sender, { nonce: userContext.vars.$loopCount });
    }
    catch (e) { } // The error can happen if transaction pool is full, so we just ignore it and keep going
    return done();
}

/// Deploy ERC20 contract and save its address to the context. Uses the precreated account from (@createTestAccount).
async function deployInkErc20(userContext, events, done) {
    if (userContext.vars.dead) {
        return done();
    }

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
        if (userContext.vars.$loopCount > 1) {
            /// It means we are not setting the erc20 address for the first time, so we go spamming.
            await tx.signAndSend(sender, { nonce: userContext.vars.$loopCount });
            return done();
        } else {
            const unsub = await tx.signAndSend(sender, { nonce: userContext.vars.$loopCount }, signWrapper(done, userContext, (result) => {
                userContext.vars.inkErc20Address = result.contract.address.toString();
                unsub();
                done();
            }));
        }
    }
    catch (e) {
        return done();
    } // The error can happen if transaction pool is full, so we just ignore it and keep going
}


/// Transfer funds from the precreated account in (@createTestAccount) to another using ERC20 contract that was deployed in (@deployInkErc20).
async function transferErc20Funds(userContext, events, done) {
    if (userContext.vars.dead) {
        return done();
    }

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
        var transfered = false;
        var whitelisted = false;

        // Alice supposed to transfer some funds to it
        const unsub = await userContext.api.tx.balances.transfer(userContext.vars.sender.address, BigInt(1000 * 1e18)).signAndSend(alice, { nonce: -1 }, signWrapper(done, userContext, (result) => {
            transfered = true;
            unsub();
            if (transfered && whitelisted) {
                done();
            }
        }));

        // Alice supposed to whitelist the account
        const unsub2 = await userContext.api.tx.sudo.sudo(userContext.api.tx.accountFilter.addAccount(userContext.vars.sender.address)).signAndSend(alice, { nonce: -1 }, signWrapper(done, userContext, (result) => {
            whitelisted = true;
            unsub2();
            if (transfered && whitelisted) {
                done();
            }
        }));
    } catch (e) {
        userContext.vars.dead = true;
        return done(); // Well, unlucky, another one will be created
    }
}

/// Return funds from the spam account to Alice.
async function returnFunds(userContext, events, done) {
    if (userContext.vars.dead) {
        return done();
    }

    const sender = userContext.vars.sender;
    const system_account = await userContext.api.query.system.account(sender.address);
    const balance = system_account.data.free.toBigInt();

    const alice = userContext.vars.alice;

    if (balance <= BigInt(1e18)) {
        return done();
    }

    try {
        /// -1e18 to cover the tx fee
        const unsub = await userContext.api.tx.balances.transfer(alice, balance - BigInt(1e18)).signAndSend(sender, { nonce: -1 }, signWrapper(done, userContext, (result) => {
            unsub();
            done();
        }));
    }
    catch (e) {
        return await returnFunds(userContext, events, done) // Keep trying
    } // The error can happen if transaction pool is full, so we just ignore it and keep going
}