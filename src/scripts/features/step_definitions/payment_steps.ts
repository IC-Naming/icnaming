import "~/setup";
import {Given, Then, When} from "@cucumber/cucumber";
import {icnaming_ledger} from "~/declarations/icnaming_ledger";
import {createLedger, ledger} from "~/declarations/ledger";
import {toHexString, toICPe8s} from "~/utils/convert";
import {
    get_quota_order_payment_receiver_subaccount_id,
    get_quota_order_payment_refund_subaccount_id
} from "~/canisters/icnaming_ledger";
import {
    AddPaymentResponse,
    RefundPaymentResponse,
    VerifyPaymentResponse
} from "~/declarations/icnaming_ledger/icnaming_ledger.did";
import {assert, expect} from 'chai';
import {reinstall_all} from "../../src/tasks"
import {identities} from "~/utils/identity";
import {get_balance_e8s, set_balance_to, transfer_to} from "./utils";
import {Tokens} from "~/declarations/ledger/ledger.did";
import logger from "node-color-log";


let global_payment_result: AddPaymentResponse;
let global_verify_payment_response: VerifyPaymentResponse;
let global_refund_payment_response: RefundPaymentResponse;
let global_sync_icp_payment_response: VerifyPaymentResponse;


const quota_order_payment_receiver_account_id = get_quota_order_payment_receiver_subaccount_id();
const quota_order_payment_refund_subaccount_id = get_quota_order_payment_refund_subaccount_id();

Given(/^Reinstall all canisters$/,
    async function () {
        await reinstall_all({
            build: false,
            init: true,
            canisters: {
                ledger: true,
                icnaming_ledger: true,
            }
        });
    });


Given(/^Create a payment with amount "([^"]*)"$/,
    async function (amount: string) {
        // add payment
        global_payment_result = await icnaming_ledger.add_payment({
            amount: {
                e8s: toICPe8s(amount)
            },
            created_remark: "payment_with_one_pay",
        });
        console.info(`a payment created with result: ${JSON.stringify(global_payment_result)}`);

    });
When(/^Transfer to icnaming ledger account with memo "([^"]*)", amount "([^"]*)"$/,
    async function (memo: string, amount: string) {

        logger.debug(`transfer to ${toHexString(Uint8Array.from(quota_order_payment_receiver_account_id))} with memo: ${memo}, amount: ${amount}`);
        // transfer payment
        const transfer_result = await ledger.transfer({
            amount: {
                e8s: toICPe8s(amount)
            },
            memo: toICPe8s(memo),
            to: quota_order_payment_receiver_account_id,
            fee: {
                e8s: BigInt(10_000),
            },
            created_at_time: [],
            from_subaccount: []
        });

        console.info(`transfer payment with result: ${JSON.stringify(transfer_result)}`);

    });
Then(/^Payment status is "([^"]*)"$/,
    function (status: string) {
        logger.debug(`payment status is ${JSON.stringify(global_verify_payment_response)}`);
        expect(status in global_verify_payment_response).to.be.true;
    });
Then(/^Payment received_amount is "([^"]*)"$/,
    function (received_amount: string) {
        if ('NeedMore' in global_verify_payment_response) {
            assert.ok(global_verify_payment_response.NeedMore.received_amount.e8s === toICPe8s(received_amount),
                `received_amount is ${global_verify_payment_response.NeedMore.received_amount.e8s}`);
        } else {
            assert.fail(`verify_payment_response is not a NeedMore object: ${JSON.stringify(global_verify_payment_response)}`);
        }
    });
Then(/^Verify payment$/,
    async function () {
        global_verify_payment_response = await icnaming_ledger.verify_payment({
            payment_id: global_payment_result.payment_id,
        });
    });
Given(/^User "([^"]*)" balance is set to be "([^"]*)"$/, async function (user: string, amount: string) {
    await set_balance_to(identities.get_identity_info(user).account_id_bytes, amount)
});
Given(/^ICNaming ledger receiver subaccount balance is set to be "([^"]*)"$/,
    async function (amount: string) {
        await set_balance_to(quota_order_payment_receiver_account_id, amount)
    });
Given(/^ICNaming ledger refund subaccount balance is set to be "([^"]*)"$/,
    async function (amount: string) {
        await set_balance_to(quota_order_payment_refund_subaccount_id, amount)
    });
Given(/^User "([^"]*)" transfer to icnaming ledger account with memo "([^"]*)", amount "([^"]*)"$/,
    async function (user: string,
                    memo: string,
                    amount: string) {
        let ledger = createLedger(identities.get_identity_info(user));
        await transfer_to(ledger, quota_order_payment_receiver_account_id, toICPe8s(amount), global_payment_result.memo);
    });
When(/^Refund last payment$/, async function () {
    global_refund_payment_response = await icnaming_ledger.refund_payment({
        payment_id: global_payment_result.payment_id,
    });
});
Then(/^Refund response status is "([^"]*)"$/,
    function (status: string) {
        if (status in global_refund_payment_response) {
            console.info(`refund payment with result: ${JSON.stringify(global_refund_payment_response)}`);
        } else {
            assert.fail(`refund_payment_result status is expected, object: ${JSON.stringify(global_refund_payment_response)}`);
        }
    });
Then(/^ICNaming ledger refund subaccount balance is "([^"]*)"$/,
    async function (balance: string) {
        let balance_result: Tokens = await ledger.account_balance({
            account: quota_order_payment_refund_subaccount_id,
        });
        assert.ok(balance_result.e8s === toICPe8s(balance), `balance is ${balance_result.e8s}`);
    });
Then(/^ICNaming ledger receiver subaccount balance is "([^"]*)"$/,
    async function (balance: string) {
        let balance_result: Tokens = await ledger.account_balance({
            account: quota_order_payment_receiver_account_id,
        });
        assert.ok(balance_result.e8s === toICPe8s(balance), `balance is ${balance_result.e8s}`);
    });
Then(/^User "([^"]*)" balance is "([^"]*)"$/,
    async function (user: string, balance: string) {
        let balance_e8s = await get_balance_e8s(user);
        assert.ok(balance_e8s === toICPe8s(balance), `balance is ${balance_e8s}`);
    });
Then(/^Verify payment with "([^"]*)" result$/,
    async function (status: string) {
        const verify_payment_result = await icnaming_ledger.verify_payment({
            payment_id: global_payment_result.payment_id,
        });
        assert.ok(status in verify_payment_result, `verify_payment_result is ${JSON.stringify(verify_payment_result)}`);
    });
Given(/^ICNaming ledger refund subaccount balance is topped up with "([^"]*)"$/,
    async function (amount: string) {
        await transfer_to(ledger, quota_order_payment_refund_subaccount_id, toICPe8s(amount), 1n);
    });
Then(/^Sync ICP payment with block height "([^"]*)"$/,
    async function (block_height: string) {
        global_sync_icp_payment_response = await icnaming_ledger.sync_icp_payment({
            block_height: BigInt(block_height),
        })
    });
Then(/^Sync ICP payment status is "([^"]*)"$/,
    function (status: string) {
        assert.ok(status in global_sync_icp_payment_response, `sync_icp_payment_result is ${JSON.stringify(global_sync_icp_payment_response)}`);
    });
Then(/^Sync ICP payment received_amount is "([^"]*)"$/,
    function (amount: string) {
        if ('NeedMore' in global_sync_icp_payment_response) {
            expect(global_sync_icp_payment_response.NeedMore.received_amount.e8s).to.equal(toICPe8s(amount));
        } else {
            assert.fail(`global_verify_payment_response is not a NeedMore object: ${JSON.stringify(global_sync_icp_payment_response)}`);
        }
    });
