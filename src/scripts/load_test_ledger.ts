import "~/setup"
import {canister, convert, identity} from "~/utils";
import {createActor} from "~/declarations/ledger";
import {AccountIdentifier, SubAccount, TransferResult} from "~/declarations/ledger/ledger.did";
import logger from "node-color-log";


// Initialize an identity from the secret key
const defaultIdentity = identity.load("default");

const name = "ledger";

const ledgerId = canister.get_id(name);
const defaultActor = createActor(ledgerId, {
    agentOptions: {
        host: "http://127.0.0.1:8000",
        identity: defaultIdentity,
    },
});

const defaultPrincipal = defaultIdentity.getPrincipal();
const defaultAccountInBytes = Array.from(convert.principalToAccountIDInBytes(defaultPrincipal));

const subaccount1_in_bytes = (() => {
    const subAccount = new Uint8Array(32).fill(0);
    subAccount[0] = 1;
    return subAccount;
})();

const subaccount1_id_in_bytes = Array.from(convert.principalToAccountIDInBytes(defaultPrincipal, subaccount1_in_bytes));

const subaccount2_in_bytes = (() => {
    const subAccount = new Uint8Array(32).fill(0);
    subAccount[0] = 2;
    return subAccount;
})();

const subaccount2_id_in_bytes = Array.from(convert.principalToAccountIDInBytes(defaultPrincipal, subaccount2_in_bytes));


const add_some_data = async () => {
    const transfer_core = async (from_subaccount: [] | [SubAccount], to: AccountIdentifier, amount: bigint, memo: bigint, fee: bigint) => {
        const result: TransferResult = await defaultActor.transfer({
            amount: {
                e8s: amount
            },
            fee: {
                e8s: fee
            },
            memo,
            from_subaccount,
            to,
            created_at_time: []
        });

        if ("Ok" in result) {
            logger.debug(`Transfer from ${from_subaccount} to ${to} with amount ${amount} and fee ${fee} succeeded`);
        } else if ("Err" in result) {
            logger.debug(result.Err);
        }
    }

    const transfer = async (to: AccountIdentifier, amount: bigint, memo: bigint, subaccount?: [SubAccount]) => {
        if (subaccount) {
            await transfer_core(subaccount, to, amount, memo, BigInt(10_000));
        } else {
            await transfer_core([], to, amount, memo, BigInt(10_000));
        }
    }


    const get_balance = async (account: number[]) => {
        const balance = await defaultActor.account_balance({account});
        logger.debug(`balance: ${balance.e8s}`);
    }

    await get_balance(defaultAccountInBytes);
    await get_balance(subaccount1_id_in_bytes);
    await get_balance(subaccount2_id_in_bytes);
    // transfer between subaccounts for 5000 times
    for (let i = 0; i < 5000; i++) {
        await transfer(subaccount1_id_in_bytes, BigInt(1_000_000), BigInt(i), [Array.from(subaccount2_in_bytes)]);
        await transfer(subaccount2_id_in_bytes, BigInt(1_000_000), BigInt(i), [Array.from(subaccount1_in_bytes)]);
    }
    await get_balance(defaultAccountInBytes);
    await get_balance(subaccount1_id_in_bytes);
    await get_balance(subaccount2_id_in_bytes);
}

(async () => {
    await add_some_data();
})().then(() => {
    logger.info("done");
}).catch((e) => {
    logger.debug(e);
})
