import {DataTable, Given, Then, When} from "@cucumber/cucumber";
import {ledger} from "~/declarations/ledger";
import {icnaming_ledger} from "~/declarations/icnaming_ledger";
import {assert_remote_result, ledger_transfer_to} from "./utils";
import {
    get_quota_order_payment_receiver_subaccount_id,
    get_quota_order_payment_refund_subaccount_id
} from "~/canisters/icnaming_ledger";
import {hexToBytes, toICPe8s} from "~/utils/convert";
import {expect} from "chai";

Given(/^ICNaming Ledger RECEIVE_SUBACCOUNT have balance to "([^"]*)"$/,
    async function (icp: string) {
        await ledger_transfer_to(ledger, get_quota_order_payment_receiver_subaccount_id(), toICPe8s(icp), 0n);
    });
Given(/^ICNaming Ledger REFUND_SUBACCOUNT have balance to "([^"]*)"$/,
    async function (icp: string) {
        await ledger_transfer_to(ledger, get_quota_order_payment_refund_subaccount_id(), toICPe8s(icp), 0n);
    });
When(/^Withdraw ICP from ICNaming Ledger RECEIVE_SUBACCOUNT with "([^"]*)"$/,
    async function (icp: string) {
        let result = await icnaming_ledger.withdraw_icp(0x11, toICPe8s(icp));
        assert_remote_result(result)
    });
When(/^Withdraw ICP from ICNaming Ledger REFUND_SUBACCOUNT with "([^"]*)"$/,
    async function (icp: string) {
        let result = await icnaming_ledger.withdraw_icp(0x12, toICPe8s(icp));
        assert_remote_result(result)
    });

Then(/^ICP Receiver account balance is "([^"]*)"$/,
    async function (icp: string) {
        let result = await ledger.account_balance({
            account: hexToBytes("63c0f188d4632e9eed8ceab624461a796b295efac6d7ecb66dfbbf17561a2362")
        });
        expect(result.e8s).to.equal(toICPe8s(icp));
    });
Then(/^ICNaming Ledger RECEIVE_SUBACCOUNT balance is "([^"]*)"$/,
    async function (icp: string) {
        let result = await ledger.account_balance({
            account: get_quota_order_payment_receiver_subaccount_id()
        });
        expect(result.e8s).to.equal(toICPe8s(icp));
    });
Then(/^ICNaming Ledger REFUND_SUBACCOUNT balance is "([^"]*)"$/,
    async function (icp: string) {
        let result = await ledger.account_balance({
            account: get_quota_order_payment_refund_subaccount_id()
        });
        expect(result.e8s).to.equal(toICPe8s(icp));
    });
