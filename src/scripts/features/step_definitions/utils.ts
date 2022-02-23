import "~/setup";
import {ledger} from "../../src/scripts/declarations/ledger";
import {_SERVICE as ledgerActor, Tokens} from "../../src/scripts/declarations/ledger/ledger.did";
import {Given, Then, When} from "@cucumber/cucumber";
import {toICPe8s} from "~/utils/convert";
import {identities} from "~/utils/identity";
import {reinstall_all} from "../../src/tasks";
import {expect} from "chai";
import {canister} from "~/utils";


Then(/^Sleep for "([^"]*)" secs.$/, async function (sec: string) {
    // sleep for secs
    await new Promise(resolve => setTimeout(resolve, parseFloat(sec) * 1000));
});

export const set_balance_to = async (to: Array<number>, balance: string | BigInt): Promise<void> => {
    let balance_bigint;
    if (typeof balance === "string") {
        balance_bigint = toICPe8s(balance);
    } else {
        balance_bigint = balance;
    }
    await transfer_to(ledger, to, balance_bigint, BigInt(0));
}

export const transfer_to = async (actor: ledgerActor, to: Array<number>, balance_bigint: bigint, memo: bigint): Promise<void> => {
    if (balance_bigint <= 0n) {
        return;
    }
    let transfer_result = await actor.transfer({
        amount: {
            e8s: balance_bigint
        },
        memo: memo,
        to: to,
        fee: {
            e8s: BigInt(10_000),
        },
        created_at_time: [],
        from_subaccount: []
    });
    if ('Err' in transfer_result) {
        throw new Error(`Failed to transfer to ${balance_bigint}, error: ${JSON.stringify(transfer_result.Err)}`);
    }
    console.info(`Transferred successfully with blockHeight: ${transfer_result.Ok}`);
}

export const get_balance_e8s = async (user: string): Promise<BigInt> => {
    const account_id = identities.get_identity_info(user).account_id_bytes;
    let balance_result: Tokens = await ledger.account_balance({
        account: account_id,
    });
    return balance_result.e8s;
}

export const reinstall_canisters = async (names: string[]): Promise<void> => {
    let canisters = {};
    for (const name of names) {
        canisters[name] = true;
    }

    console.info(`Reinstalling canisters: ${JSON.stringify(canisters)}`);

    await reinstall_all({
        build: false,
        init: true,
        canisters: canisters
    });
}

export const assert_remote_result = (result: any, status: string) => {
    if (status === 'Ok') {
        expect('Ok' in result).to.be.true;
    } else {
        if ('Err' in result) {
            expect(result.Err.message).to.be.equal(status);
        } else {
            expect.fail(`Expected to be error but found ${JSON.stringify(result)}`);
        }
    }
}

Given(/^Reinstall canisters$/,
    async function (data) {
        let target_canisters = data.hashes();
        let names: string[] = [];
        for (const item of target_canisters) {
            names.push(item.name);
        }
        await reinstall_canisters(names);
    });
When(/^canister "([^"]*)" is down$/, async function (canister_name: string) {
    await canister.uninstall_code(canister_name);
});

