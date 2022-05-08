import "~/setup";
import {DataTable, Given, Then, When} from "@cucumber/cucumber";
import {createRegistrar, registrar} from "~/declarations/registrar";
import {registrar_control_gateway} from "~/declarations/registrar_control_gateway";
import {createLedger, ledger} from "~/declarations/ledger";
import {assert, expect} from 'chai';
import {reinstall_all} from "~/../tasks"
import {
    BooleanActorResponse as RefundResult,
    BooleanActorResponse as AvailableResult,
    BooleanActorResponse as RegisterWithQuotaResult,
    BooleanActorResponse as TransferResult,
    BooleanActorResponse as TransferFromResult,
    BooleanActorResponse as TransferByAdminResult,
    GetNameOrderResponse,
    QuotaType,
    Stats,
    SubmitOrderActorResponse as SubmitOrderResult,
} from "~/declarations/registrar/registrar.did";
import {toICPe8s,} from "~/utils/convert";
import {get_principal, identities} from "~/utils/identity";
import {OptionalResult, Result} from "~/utils/Result";
import {assert_remote_result} from "./utils";
import logger from "node-color-log";
import fs from "fs";
import {
    AssignNameResponse,
    ImportQuotaResponse
} from "~/declarations/registrar_control_gateway/registrar_control_gateway.did";

let global_submit_order_result: SubmitOrderResult;
let global_refund_response: RefundResult;
let global_available_response: AvailableResult;
let global_register_with_quota_response: RegisterWithQuotaResult;
let global_quota_import_response: ImportQuotaResponse;
let global_stats_result: Stats;
let global_assign_name_result: AssignNameResponse;
let global_transfer_result: TransferResult;
let global_transfer_from_result: TransferFromResult;
let global_transfer_by_admin_result: TransferByAdminResult;

async function submit_order(user: string | null, name: string, years: string) {
    let actor;
    if (user) {
        let identityInfo = identities.get_identity_info(user);
        actor = createRegistrar(identityInfo);
    } else {
        actor = registrar;
    }
    const call = actor.submit_order({
        name,
        years: parseInt(years)
    });

    global_submit_order_result = await call;
}


async function pay_to_pending_order(user: string | null, amount: string) {
    let current_registrar;
    let current_ledger;
    if (user) {
        let identityInfo = identities.get_identity_info(user);
        current_registrar = createRegistrar(identityInfo);
        current_ledger = createLedger(identityInfo);
    } else {
        current_registrar = registrar;
        current_ledger = ledger;
    }
    let optionalResult: OptionalResult<GetNameOrderResponse> = new OptionalResult(current_registrar.get_pending_order());
    const order = await optionalResult.unwrap();
    const e8s = toICPe8s(amount);
    console.debug(`Pay for order: ${JSON.stringify(order)} with amount: ${e8s}`);
    let transfer_result = await current_ledger.transfer({
        amount: {
            e8s: BigInt(e8s)
        },
        memo: toICPe8s(order.payment_memo.ICP.toString()),
        to: order.payment_account_id,
        fee: {
            e8s: BigInt(10_000),
        },
        created_at_time: [],
        from_subaccount: []
    });
    if ('Err' in transfer_result) {
        assert(false, transfer_result.Err.message);
    }
}

async function ensure_no_pending_order(user: string | null) {
    let actor;
    if (user) {
        let identityInfo = identities.get_identity_info(user);
        actor = createRegistrar(identityInfo);
    } else {
        actor = registrar;
    }
    const get_pending_order_result = await actor.get_pending_order();
    if ('Err' in get_pending_order_result) {
        assert(false, get_pending_order_result.Err.message);
    } else {
        assert(get_pending_order_result.Ok.length === 0, 'Pending order found');
    }
}

async function ensure_pending_order(user: string | null, name: string, years: string, status: string | null) {
    let actor;
    if (user) {
        let identityInfo = identities.get_identity_info(user);
        actor = createRegistrar(identityInfo);
    } else {
        actor = registrar;
    }
    let order = await new OptionalResult(actor.get_pending_order()).unwrap() as GetNameOrderResponse;
    logger.debug(`Order: ${JSON.stringify(order)}`);
    assert(order.name === name, 'Name not match');
    assert(order.years === parseInt(years), 'Years not match');
    if (status) {
        expect(status in order.status).to.be.true;
    }
}

function diff_less_than(a: bigint, b: bigint, diff: bigint): boolean {
    if (a > b) {
        return (a - b) < diff;
    } else {
        return b - a < diff;
    }
}

function is_around_to_date(value: bigint, diff_year: number): boolean {
    let target = now_add_years(diff_year);
    return diff_less_than(value, target, BigInt(60000));
}

function now_add_years(years: number): bigint {
    let now = BigInt(Date.now());
    return now + BigInt(years * 365 * 24 * 60 * 60 * 1000);
}

Given(/^Reinstall registrar related canisters$/,
    async function () {
        await reinstall_all({
            build: false,
            init: true,
            canisters: {
                ledger: true,
                icnaming_ledger: false,
                registrar: true,
                registry: true,
                resolver: true,
                cycles_minting: true,
                registrar_control_gateway: true,
            }
        });
    });
When(/^I submit a order to register name "([^"]*)" for "([^"]*)" years$/,
    async function (name: string, years: string) {
        await submit_order(null, name, years);
    });
Then(/^I found my pending order with "([^"]*)" for "([^"]*)" years$/,
    async function (name: string, years: string) {
        await ensure_pending_order(null, name, years, null);
    });
When(/^I cancel my pending order$/,
    async function () {
        let cancel_result = await registrar.cancel_order();
        if ('Err' in cancel_result) {
            assert(false, cancel_result.Err.message);
        }
    });
Then(/^I found there is no pending order$/,
    async function () {
        await ensure_no_pending_order(null);
    });
When(/^Pay for my pending order with amount "([^"]*)"$/,
    async function (amount: string) {
        await pay_to_pending_order(null, amount);
    });

Then(/^name "([^"]*)" is available$/,
    async function (name: string) {
        const available_result = await new Result(registrar.available(name)).unwrap();
        assert(available_result, 'Name not available');
    });
Given(/^User "([^"]*)" submit a order to register name "([^"]*)" for "([^"]*)" years$/,
    async function (user: string, name: string, years: string) {
        await submit_order(user, name, years);
    });
When(/^User "([^"]*)" pay for my pending order with amount "([^"]*)"$/,
    async function (user: string, amount: string) {
        await pay_to_pending_order(user, amount);
    });
Then(/^User "([^"]*)" found there is no pending order$/,
    async function (user: string) {
        await ensure_no_pending_order(user);
    });
Then(/^User "([^"]*)" found my pending order with "([^"]*)" for "([^"]*)" years, status "([^"]*)"$/,
    async function (user: string, name: string, years: string, status: string) {
        await ensure_pending_order(user, name, years, status);
    });
When(/^User "([^"]*)" refund my pending order$/,
    async function (user: string) {
        let identityInfo = identities.get_identity_info(user);
        let actor = createRegistrar(identityInfo);
        global_refund_response = await actor.refund_order();
    });
Then(/^Last refund response is "([^"]*)"$/,
    function (status: string) {
        if (status === "Ok") {
            assert(status in global_refund_response,
                `Status not match: ${JSON.stringify(global_refund_response)}`);
        } else {
            if ('Err' in global_refund_response) {
                assert(global_refund_response.Err.message === status);
            } else {
                assert(false, `Status not match: ${JSON.stringify(global_refund_response)}`);
            }
        }
    });
When(/^Check availability of "([^"]*)"$/,
    async function (name: string) {
        global_available_response = await registrar.available(name);
    });
Then(/^Check result of "([^"]*)" is '([^']*)'$/,
    async function (name: string, status: string) {
        if (status === "Ok") {
            expect(status in global_available_response)
                .to.be.true;
            assert(status in global_available_response,
                `Status not match: ${JSON.stringify(global_available_response)}`);
        } else {
            if ('Err' in global_available_response) {
                expect(global_available_response.Err.message).to.equal(status);
            } else {
                expect.fail(`Status not match: ${JSON.stringify(global_available_response)}`);
            }
        }
    });
Given(/^Name "([^"]*)" is already taken$/,
    async function (name: string) {
        const quote_type = {
            LenGte: 1,
        };
        let identityInfo = identities.main;
        let registrar = createRegistrar(identityInfo);
        await new Result(registrar.add_quota(identities.main.identity.getPrincipal(), quote_type, 1)).unwrap();
        await new Result(registrar.register_with_quota(name, quote_type)).unwrap();
    });
Then(/^get_name_expires "([^"]*)" result is about in "([^"]*)" years$/,
    async function (name: string, year: string) {
        let expired_at = await new Result(registrar.get_name_expires(name)).unwrap();
        expect(is_around_to_date(expired_at, parseInt(year))).to.be.true;
    });
Then(/^get_owner result "([^"]*)" is the same as "([^"]*)" identity$/,
    async function (name: string, user: string) {
        let owner = await new Result(registrar.get_owner(name)).unwrap();
        let identityInfo = identities.get_identity_info(user);
        expect(owner.toText()).to.equal(identityInfo.principal_text);
    });

Then(/^registrar get_details "([^"]*)" result is$/,
    async function (name: string, data) {
        let details = await new Result(registrar.get_details(name)).unwrap();
        let target = data.rowsHash();
        console.info(`details: ${JSON.stringify(details)}`);

        let identityInfo = identities.get_identity_info(target.owner);
        expect(details.owner.toText()).to.equal(identityInfo.principal_text);

        expect(details.name).to.equal(target.name);
        expect(is_around_to_date(details.expired_at, parseInt(target.expired_at))).to.be.true;
        expect(is_around_to_date(details.created_at, parseInt(target.created_at))).to.be.true;
    });
When(/^Update quota as follow operations$/,
    async function (data) {
        let operations: { operation: string, user: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes();

        // create actor from main as administrator
        let identityInfo = identities.main;
        let registrar = createRegistrar(identityInfo);

        for (const op of operations) {
            let quota_type = {};
            quota_type[op.quota_type1] = parseInt(op.quota_type2);
            let user_principal = identities.get_principal(op.user);
            if (op.operation === "add") {
                await new Result(registrar.add_quota(user_principal, quota_type as QuotaType, parseInt(op.value))).unwrap();
            } else {
                await new Result(registrar.sub_quota(user_principal, quota_type as QuotaType, parseInt(op.value))).unwrap();
            }
        }
    });
Then(/^User quota status should be as below$/,
    async function (data) {
        let items: { user: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes();

        // create actor from main as administrator
        let identityInfo = identities.main;
        let registrar = createRegistrar(identityInfo);

        for (const item of items) {
            let quota_type = {};
            quota_type[item.quota_type1] = parseInt(item.quota_type2);
            let user_principal = identities.get_principal(item.user)
            let quota_value = await new Result(registrar.get_quota(user_principal, quota_type as QuotaType)).unwrap();
            expect(quota_value).to.equal(parseInt(item.value));
        }
    });
When(/^Do quota transfer as below$/,
    async function (data: DataTable) {
        let items: { from: string, to: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes();

        for (const item of items) {
            let quota_type = {};
            quota_type[item.quota_type1] = parseInt(item.quota_type2);
            let to_principal = get_principal(item.to);
            let registrar = createRegistrar(identities.get_identity_info(item.from));
            await registrar.transfer_quota(to_principal, quota_type as QuotaType, parseInt(item.value));
        }
    });
Given(/^Some users already have some quotas$/,
    async function (data) {
        let items: { user: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes();

        // create actor from main as administrator
        let identityInfo = identities.main;
        let registrar = createRegistrar(identityInfo);

        for (const item of items) {
            let quota_type = {};
            quota_type[item.quota_type1] = parseInt(item.quota_type2);
            let user_principal = identities.get_identity_info(item.user).identity.getPrincipal();
            await new Result(registrar.add_quota(user_principal, quota_type as QuotaType, parseInt(item.value))).unwrap();
        }
    });

When(/^Do quota transfer_from_quota as below by admin$/,
    async function (data: DataTable) {
        let items: { from: string, to: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes();
        for (const item of items) {
            let quota_type = {};
            quota_type[item.quota_type1] = parseInt(item.quota_type2);
            let to_principal = get_principal(item.to);
            let from_principal = get_principal(item.from);
            await registrar.transfer_from_quota({
                from: from_principal,
                to: to_principal,
                quota_type: quota_type as QuotaType,
                diff: parseInt(item.value)
            });
        }
    });

function to_quota_type(source: string): QuotaType {
    // get number in ()
    let match = source.match(/\(([0-9]+)\)/);
    if (!match) {
        throw new Error("Invalid quota type");
    }
    let value = parseInt(match[1]);
    if (source.startsWith("LenGte")) {
        return {
            LenGte: value
        };
    } else {
        return {
            LenEq: value
        }
    }
}

When(/^User "([^"]*)" register name "([^"]*)" with quote "([^"]*)"$/,
    async function (user: string, name: string, quota_string: string) {
        let quotaType = to_quota_type(quota_string);
        let identityInfo = identities.get_identity_info(user);
        let registrar = createRegistrar(identityInfo);
        global_register_with_quota_response = await registrar.register_with_quota(name, quotaType);
    });
Then(/^Register with quota result in status '([^']*)'$/,
    async function (status: string) {
        if (status === "Ok") {
            expect('Ok' in global_register_with_quota_response).to.be.true;
        } else {
            if ('Err' in global_register_with_quota_response) {
                expect(global_register_with_quota_response.Err.message).to.equal(status);
            } else {
                throw new Error(`Register with quota result is not Err but ${JSON.stringify(global_register_with_quota_response)}`);
            }
        }
    });
When(/^User "([^"]*)" register name "([^"]*)" with quote "([^"]*)" for "([^"]*)" with "([^"]*)" years$/,
    async function (user: string, name: string, quota_string: string, user_for: string, years: string) {
        let identityInfo = identities.get_identity_info(user);
        let registrar = createRegistrar(identityInfo);

        let userForIdentityInfo = identities.get_identity_info(user_for);

        await registrar.register_for(name, userForIdentityInfo.identity.getPrincipal(), BigInt(parseInt(years)));
    });
Then(/^Order submitting result in status '([^']*)'$/,
    function (status: string) {
        assert_remote_result(global_submit_order_result, status);
    });

Then(/^I found my pending order as bellow$/,
    async function (data: DataTable
    ) {
        let registrar = createRegistrar(identities.main);
        let order: GetNameOrderResponse = await new OptionalResult(registrar.get_pending_order()).unwrap();
        let rows = data.rowsHash();
        expect(order.name).to.equal(rows.name);
        expect(order.price_icp_in_e8s).to.equal(toICPe8s(rows.price_icp_in_e8s));
        let target_type = to_quota_type(rows.quota_type);
        expect(order.quota_type).to.deep.equal(target_type);
        expect(order.years).to.equal(parseInt(rows.years));
    });
Given(/^Record payment version$/,
    async function () {
        let timer_target = createRegistrar(identities.timer_trigger);
        await timer_target.run_tasks();
        global_stats_result = await new Result(registrar.get_stats()).unwrap();
        console.info(`Record payment version: ${global_stats_result.payment_version}`);
    });
When(/^Wait for payment version increased with "([^"]*)"$/,
    async function (version_change: string) {
        let timer_target = createRegistrar(identities.timer_trigger);

        let max_retry = 20;
        let version_add = BigInt(parseInt(version_change));
        let target_version = global_stats_result.payment_version + version_add;
        for (let i = 0; i < max_retry; i++) {
            await timer_target.run_tasks();
            let stats = await new Result(registrar.get_stats()).unwrap();
            if (stats.payment_version >= target_version) {
                console.info(`Payment version increased to ${target_version}, now is ${stats.payment_version}`);
                return;
            } else {
                console.debug(`Wait for payment version increased to ${target_version}, now is ${stats.payment_version}`);
                await new Promise(resolve => setTimeout(resolve, 1000));
            }
        }

    });
When(/^admin import quota file "([^"]*)"$/,
    async function (filename: string) {
        // read file from ../../quota_import_data/filename as bytes
        let content = fs.readFileSync(`quota_import_data/${filename}`);
        global_quota_import_response = await registrar_control_gateway.import_quota(Array.from(content));
    });
Then(/^Last quota import status "([^"]*)"$/,
    function (status) {
        if ('Ok' in global_quota_import_response) {
            if (!(status in global_quota_import_response.Ok)) {
                expect.fail(`Last quota import status is not ${status} but ${JSON.stringify(global_quota_import_response.Ok)}`);
            }
        } else {
            expect.fail(`Last quota import status is not Ok but ${JSON.stringify(global_quota_import_response)}`);
        }
    });
When(/^User "([^"]*)" confirm pay order with block height "([^"]*)"$/,
    async function (user: string, block_height: string) {
        let registrar = createRegistrar(identities.get_identity_info(user));
        let result = await new Result(registrar.confirm_pay_order(BigInt(parseInt(block_height)))).unwrap();
        logger.info(`confirm pay order result: ${JSON.stringify(result)}`);
    });
Given(/^admin assign name "([^"]*)" to user "([^"]*)"$/,
    async function (name: string, user: string) {
        global_assign_name_result = await registrar_control_gateway.assign_name(name, identities.get_identity_info(user).identity.getPrincipal());
    });
Then(/^last assign name status is "([^"]*)"$/,
    function (status: string) {
        if ('Ok' in global_assign_name_result) {
            if (!(status in global_assign_name_result.Ok)) {
                expect.fail(`last assign name status is not ${status} but ${JSON.stringify(global_assign_name_result.Ok)}`);
            }
        } else {
            expect.fail(`last assign name status is not Ok but ${JSON.stringify(global_assign_name_result)}`);
        }
    });
When(/^User "([^"]*)" transfer name "([^"]*)" to User "([^"]*)"$/,
    async function (user: string, name: string, new_owner: string) {
        let registrar = createRegistrar(identities.get_identity_info(user));
        let new_owner_principal = identities.get_principal(new_owner);
        global_transfer_result = await registrar.transfer(name, new_owner_principal);
    });
Then(/^last name transfer result status is "([^"]*)"$/,
    function (status: string) {
        assert_remote_result(global_transfer_result, status);
    });
Given(/^User "([^"]*)" approve name "([^"]*)" to User "([^"]*)"$/,
    async function (user: string, name: string, to: string) {
        let registrar = createRegistrar(identities.get_identity_info(user));
        let to_principal = get_principal(to);
        await new Result(registrar.approve(name, to_principal)).unwrap();
    });
When(/^User "([^"]*)" transfer name "([^"]*)" by transfer_from$/,
    async function (user: string, name: string) {
        let registrar = createRegistrar(identities.get_identity_info(user));
        global_transfer_from_result = await registrar.transfer_from(name);
    });
Then(/^last name transfer_from result status is "([^"]*)"$/,
    async function (status: string) {
        assert_remote_result(global_transfer_from_result, status);

    });
When(/^User "([^"]*)" transfer name "([^"]*)" to user "([^"]*)"$/,
    async function (user: string, name: string, to: string) {
        let registrar = createRegistrar(identities.get_identity_info(user));
        let to_principal = get_principal(to);
        global_transfer_by_admin_result = await registrar.transfer_by_admin(name, to_principal);
    });
Then(/^last transfer_by_admin status is "([^"]*)"$/,
    function (status: string) {
        assert_remote_result(global_transfer_by_admin_result, status);
    });
Then(/^Get last registrations result is$/,
    async function (data: DataTable) {
        let items: { name: string }[] = data.hashes();
        let registrar = createRegistrar(identities.timer_trigger);
        let last_registrations = await new Result(registrar.get_last_registrations()).unwrap();
        let actual_names = last_registrations.map(r => r.name);
        // expect item and order
        for (let i = 0; i < items.length; i++) {
            expect(actual_names[i]).to.equal(items[i].name);
        }
    });