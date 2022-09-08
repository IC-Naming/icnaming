import "./setup"
import {DataTable, Given, Then, When} from '@cucumber/cucumber'
import {createRegistrar, registrar} from '~/declarations/registrar'
import {registrar_control_gateway} from '~/declarations/registrar_control_gateway'
import {createDicp, dicp} from '~/declarations/dicp'
import {assert, expect} from 'chai'
import {reinstall_all} from '~/../tasks'
import {
    BooleanActorResponse as AvailableResult,
    BooleanActorResponse as RegisterWithQuotaResult,
    BooleanActorResponse as TransferResult,
    BooleanActorResponse as TransferFromResult,
    BooleanActorResponse as TransferByAdminResult,
    BooleanActorResponse as RenewNameResult,
    QuotaType,
    GetDetailsActorResponse as RegisterWithPaymentResponse,
    GetNameStatueActorResponse,
} from '~/declarations/registrar/registrar.did'
import {identities} from '~/identityHelper'
import {Result} from '~/utils/Result'
import {assert_remote_result} from './utils'
import logger from 'node-color-log'
import fs from 'fs'
import {canister, utils} from '@deland-labs/ic-dev-kit'
import {
    AssignNameResponse,
    ImportQuotaResponse
} from '~/declarations/registrar_control_gateway/registrar_control_gateway.did'

let global_available_response: AvailableResult
let global_register_with_quota_response: RegisterWithQuotaResult
let global_quota_import_response: ImportQuotaResponse
let global_assign_name_result: AssignNameResponse
let global_transfer_result: TransferResult
let global_transfer_from_result: TransferFromResult
let global_transfer_by_admin_result: TransferByAdminResult
let global_renew_name_result: RenewNameResult
let global_get_name_status_result: GetNameStatueActorResponse
let global_register_with_payment_result: RegisterWithPaymentResponse

function diff_less_than(a: bigint, b: bigint, diff: bigint): boolean {
    if (a > b) {
        return (a - b) < diff
    } else {
        return b - a < diff
    }
}

function is_around_to_date(value: bigint, diff_year: number): boolean {
    const target = now_add_years(diff_year)
    logger.debug(`Target: ${target}, Value: ${value}`)
    return diff_less_than(value, target, BigInt(60000))
}

function now_add_years(years: number): bigint {
    const date = new Date(Date.now())
    date.setFullYear(date.getFullYear() + years)
    return BigInt(date.getTime())
}

function get_expired_at(years: number): bigint {
    const date = new Date(Date.now())
    date.setFullYear(date.getFullYear() + years)
    date.setHours(0, 0, 0, 0)
    return BigInt(date.getTime())
}

Given(/^Reinstall registrar related canisters$/,
    async function () {
        await reinstall_all({
            build: false,
            init: true,
            canisters: {
                dicp: true,
                registrar: true,
                registry: true,
                resolver: true,
                cycles_minting: true,
                registrar_control_gateway: true
            }
        })
    })
Then(/^name "([^"]*)" is available$/,
    async function (name: string) {
        const available_result = await new Result(registrar.available(name)).unwrap()
        assert(available_result, 'Name not available')
    })
When(/^Check availability of "([^"]*)"$/,
    async function (name: string) {
        global_available_response = await registrar.available(name)
    })
Then(/^Check result of "([^"]*)" is '([^']*)'$/,
    async function (name: string, status: string) {
        if (status === 'Ok') {
            expect(status in global_available_response)
                .to.be.true
            assert(status in global_available_response,
                `Status not match: ${JSON.stringify(global_available_response)}`)
        } else {
            if ('Err' in global_available_response) {
                expect(global_available_response.Err.message).to.equal(status)
            } else {
                expect.fail(`Status not match: ${JSON.stringify(global_available_response)}`)
            }
        }
    })
Given(/^Name "([^"]*)" is already taken$/,
    async function (name: string) {
        const quote_type = {
            LenGte: 1
        }
        const identityInfo = identities.main
        const registrar = createRegistrar(identityInfo)
        await new Result(registrar.add_quota(identities.main.identity.getPrincipal(), quote_type, 1)).unwrap()
        await new Result(registrar.register_with_quota(name, quote_type)).unwrap()
    })
Then(/^get_name_expires "([^"]*)" result is about in "([^"]*)" years$/,
    async function (name: string, year: string) {
        const expired_at = await new Result(registrar.get_name_expires(name)).unwrap()
        const expectedExpiredAt = get_expired_at(parseInt(year))
        expect(expired_at).eq(expectedExpiredAt)
    })
Then(/^get_owner result "([^"]*)" is the same as "([^"]*)" identity$/,
    async function (name: string, user: string) {
        const owner = await new Result(registrar.get_owner(name)).unwrap()
        const identityInfo = identities.getIdentity(user)
        expect(owner.toText()).to.equal(identityInfo.principalText)
    })

Then(/^registrar get_details "([^"]*)" result is$/,
    async function (name: string, data) {
        const details = await new Result(registrar.get_details(name)).unwrap()
        const target = data.rowsHash()
        console.info(`details: ${JSON.stringify(details)}`)

        const identityInfo = identities.getIdentity(target.owner)
        expect(details.owner.toText()).to.equal(identityInfo.principalText)

        expect(details.name).to.equal(target.name)
        const expiredAt = get_expired_at(parseInt(target.expired_at))
        expect(details.expired_at).eq(expiredAt)
        expect(is_around_to_date(details.created_at, parseInt(target.created_at))).to.be.true
    })
When(/^Update quota as follow operations$/,
    async function (data) {
        const operations: { operation: string, user: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes()

        // create actor from main as administrator
        const identityInfo = identities.main
        const registrar = createRegistrar(identityInfo)

        for (const op of operations) {
            const quota_type = {}
            quota_type[op.quota_type1] = parseInt(op.quota_type2)
            const user_principal = identities.getPrincipal(op.user)
            if (op.operation === 'add') {
                await new Result(registrar.add_quota(user_principal, quota_type as QuotaType, parseInt(op.value))).unwrap()
            } else {
                await new Result(registrar.sub_quota(user_principal, quota_type as QuotaType, parseInt(op.value))).unwrap()
            }
        }
    })
Then(/^User quota status should be as below$/,
    async function (data) {
        const items: { user: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes()

        // create actor from main as administrator
        const identityInfo = identities.main
        const registrar = createRegistrar(identityInfo)

        for (const item of items) {
            const quota_type = {}
            quota_type[item.quota_type1] = parseInt(item.quota_type2)
            const user_principal = identities.getPrincipal(item.user)
            const quota_value = await new Result(registrar.get_quota(user_principal, quota_type as QuotaType)).unwrap()
            expect(quota_value).to.equal(parseInt(item.value))
        }
    })
When(/^Do quota transfer as below$/,
    async function (data: DataTable) {
        const items: { from: string, to: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes()

        for (const item of items) {
            const quota_type = {}
            quota_type[item.quota_type1] = parseInt(item.quota_type2)
            const to_principal = identities.getPrincipal(item.to)
            const registrar = createRegistrar(identities.getIdentity(item.from))
            await registrar.transfer_quota(to_principal, quota_type as QuotaType, parseInt(item.value))
        }
    })
Given(/^Some users already have some quotas$/,
    async function (data) {
        const items: { user: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes()

        // create actor from main as administrator
        const identityInfo = identities.main
        const registrar = createRegistrar(identityInfo)

        for (const item of items) {
            const quota_type = {}
            quota_type[item.quota_type1] = parseInt(item.quota_type2)
            const user_principal = identities.getPrincipal(item.user)
            await new Result(registrar.add_quota(user_principal, quota_type as QuotaType, parseInt(item.value))).unwrap()
        }
    })

When(/^Do quota transfer_from_quota as below by admin$/,
    async function (data: DataTable) {
        const items: { from: string, to: string, quota_type1: string, quota_type2: string, value: string }[] = data.hashes()
        for (const item of items) {
            const quota_type = {}
            quota_type[item.quota_type1] = parseInt(item.quota_type2)
            const to_principal = identities.getPrincipal(item.to)
            const from_principal = identities.getPrincipal(item.from)
            await registrar.transfer_from_quota({
                from: from_principal,
                to: to_principal,
                quota_type: quota_type as QuotaType,
                diff: parseInt(item.value)
            })
        }
    })

function to_quota_type(source: string): QuotaType {
    // get number in ()
    const match = source.match(/\(([0-9]+)\)/)
    if (!match) {
        throw new Error('Invalid quota type')
    }
    const value = parseInt(match[1])
    if (source.startsWith('LenGte')) {
        return {
            LenGte: value
        }
    } else {
        return {
            LenEq: value
        }
    }
}

When(/^User "([^"]*)" register name "([^"]*)" with quote "([^"]*)"$/,
    async function (user: string, name: string, quota_string: string) {
        const quotaType = to_quota_type(quota_string)
        const identityInfo = identities.getIdentity(user)
        const registrar = createRegistrar(identityInfo)
        global_register_with_quota_response = await registrar.register_with_quota(name, quotaType)
    })
Then(/^Register with quota result in status '([^']*)'$/,
    async function (status: string) {
        if (status === 'Ok') {
            expect('Ok' in global_register_with_quota_response).to.be.true
        } else {
            if ('Err' in global_register_with_quota_response) {
                expect(global_register_with_quota_response.Err.message).to.equal(status)
            } else {
                throw new Error(`Register with quota result is not Err but ${JSON.stringify(global_register_with_quota_response)}`)
            }
        }
    })
When(/^User "([^"]*)" register name "([^"]*)" with quote "([^"]*)" for "([^"]*)" with "([^"]*)" years$/,
    async function (user: string, name: string, quota_string: string, user_for: string, years: string) {
        const identityInfo = identities.getIdentity(user)
        const registrar = createRegistrar(identityInfo)

        const userForIdentityInfo = identities.getIdentity(user_for)

        await registrar.register_for(name, userForIdentityInfo.identity.getPrincipal(), BigInt(parseInt(years)))
    })

When(/^admin import quota file "([^"]*)"$/,
    async function (filename: string) {
        // read file from ../../quota_import_data/filename as bytes
        const content = fs.readFileSync(`quota_import_data/${filename}`)
        global_quota_import_response = await registrar_control_gateway.import_quota(Array.from(content))
    })
Then(/^Last quota import status "([^"]*)"$/,
    function (status) {
        if ('Ok' in global_quota_import_response) {
            if (!(status in global_quota_import_response.Ok)) {
                expect.fail(`Last quota import status is not ${status} but ${JSON.stringify(global_quota_import_response.Ok)}`)
            }
        } else {
            expect.fail(`Last quota import status is not Ok but ${JSON.stringify(global_quota_import_response)}`)
        }
    })

Given(/^admin assign name "([^"]*)" to user "([^"]*)"$/,
    async function (name: string, user: string) {
        global_assign_name_result = await registrar_control_gateway.assign_name(name, identities.getIdentity(user).identity.getPrincipal())
    })
Then(/^last assign name status is "([^"]*)"$/,
    function (status: string) {
        if ('Ok' in global_assign_name_result) {
            if (!(status in global_assign_name_result.Ok)) {
                expect.fail(`last assign name status is not ${status} but ${JSON.stringify(global_assign_name_result.Ok)}`)
            }
        } else {
            expect.fail(`last assign name status is not Ok but ${JSON.stringify(global_assign_name_result)}`)
        }
    })
When(/^User "([^"]*)" transfer name "([^"]*)" to User "([^"]*)"$/,
    async function (user: string, name: string, new_owner: string) {
        const registrar = createRegistrar(identities.getIdentity(user))
        const new_owner_principal = identities.getPrincipal(new_owner)
        global_transfer_result = await registrar.transfer(name, new_owner_principal)
    })
Then(/^last name transfer result status is "([^"]*)"$/,
    function (status: string) {
        assert_remote_result(global_transfer_result, status)
    })
Given(/^User "([^"]*)" approve name "([^"]*)" to User "([^"]*)"$/,
    async function (user: string, name: string, to: string) {
        const registrar = createRegistrar(identities.getIdentity(user))
        const to_principal = identities.getPrincipal(to)
        await new Result(registrar.approve(name, to_principal)).unwrap()
    })
When(/^User "([^"]*)" transfer name "([^"]*)" by transfer_from$/,
    async function (user: string, name: string) {
        const registrar = createRegistrar(identities.getIdentity(user))
        global_transfer_from_result = await registrar.transfer_from(name)
    })
Then(/^last name transfer_from result status is "([^"]*)"$/,
    async function (status: string) {
        assert_remote_result(global_transfer_from_result, status)
    })
When(/^User "([^"]*)" transfer name "([^"]*)" to user "([^"]*)"$/,
    async function (user: string, name: string, to: string) {
        const registrar = createRegistrar(identities.getIdentity(user))
        const to_principal = identities.getPrincipal(to)
        global_transfer_by_admin_result = await registrar.transfer_by_admin(name, to_principal)
    })
Then(/^last transfer_by_admin status is "([^"]*)"$/,
    function (status: string) {
        assert_remote_result(global_transfer_by_admin_result, status)
    })
Then(/^Get last registrations result is$/,
    async function (data: DataTable) {
        const items: { name: string }[] = data.hashes()
        const registrar = createRegistrar(identities.getIdentity("timer_trigger"))
        const last_registrations = await new Result(registrar.get_last_registrations()).unwrap()
        const actual_names = last_registrations.map(r => r.name)
        // expect item and order
        for (let i = 0; i < items.length; i++) {
            expect(actual_names[i]).to.equal(items[i].name)
        }
    })
When(/^User "([^"]*)" reclaim name "([^"]*)"$/,
    async function (user: string, name: string) {
        const registrar = createRegistrar(identities.getIdentity(user))
        const result = await registrar.reclaim_name(name)
        logger.debug(`reclaim_name result: ${JSON.stringify(result)}`)
    })

interface BatchTransferQuotaDetails {
    to: string;
    quota_type1: string;
    quota_type2: string;
    diff: string;
}

When(/^User "([^"]*)" transfer quota as below by batch$/,
    async function (user: string, data: DataTable) {
        const items: BatchTransferQuotaDetails[] = data.hashes()
        const registrar = createRegistrar(identities.getIdentity(user))
        const request_items = items.map(item => {
            const quota_type = {}
            quota_type[item.quota_type1] = parseInt(item.quota_type2)
            return {
                to: identities.getPrincipal(item.to),
                quota_type: quota_type as QuotaType,
                diff: parseInt(item.diff)
            }
        })
        const result = await registrar.batch_transfer_quota({
            items: request_items
        })
        logger.debug(`batch_transfer_quota result: ${JSON.stringify(result)}`)
    })
When(/^User "([^"]*)" renew name "([^"]*)" for "([^"]*)" years and pay "([^"]*)"$/,
    async function (user: string, name: string, years: string, approve_amount: string) {
        const registrar = createRegistrar(identities.getIdentity(user))
        const dicp = createDicp(identities.getIdentity(user))
        const amount = utils.toICPe8s(approve_amount)
        {
            const result = await dicp.approve([], canister.get_id('registrar'), amount, [])
            logger.debug(`approve result: ${JSON.stringify(result)}`)
        }
        {
            const result = await registrar.renew_name({
                name,
                years: parseInt(years),
                approve_amount: amount
            })
            global_renew_name_result = result
            logger.debug(`renew_name result: ${JSON.stringify(result)}`)
        }
    })
Then(/^Last renew name status is "([^"]*)"$/,
    function (status: string) {
        assert_remote_result(global_renew_name_result, status)
    })

interface ImportNameItem {
    name: string,
    owner: string,
    years: string
}

When(/^Admin import names as following$/,
    async function (data: DataTable) {
        const items: ImportNameItem[] = data.hashes()
        const result = await registrar.import_registrations({
            items: items.map(item => {
                return {
                    name: item.name,
                    owner: identities.getPrincipal(item.owner),
                    years: parseInt(item.years)
                }
            })
        })
        logger.info(`import_registrations result: ${JSON.stringify(result)}`)
    })
When(/^get_name_status "([^"]*)" result$/,
    async function (name: string) {
        global_get_name_status_result = await registrar.get_name_status(name);
    });
Then(/^Check get_name_status is$/,
    async function (data: DataTable) {
        const target = data.rowsHash()
        if ('Ok' in global_get_name_status_result) {
            const actual = global_get_name_status_result.Ok
            expect(actual.kept.toString()).to.equal(target.kept, "kept")
            expect(actual.registered.toString()).to.equal(target.registered, "registered")
            expect(actual.available.toString()).to.equal(target.available, "keavailablept")
        } else {
            expect.fail(`get_name_status result is not Ok: ${JSON.stringify(global_get_name_status_result)}`)
        }
    });
When(/^User "([^"]*)" register name "([^"]*)" for "([^"]*)" years and pay "([^"]*)"$/,
    async function (user: string, name: string, years: string, approve_amount: string) {
        const registrar = createRegistrar(identities.getIdentity(user))
        const dicp = createDicp(identities.getIdentity(user))
        const amount = utils.toICPe8s(approve_amount)
        await dicp.approve([], canister.get_id('registrar'), amount, [])
        global_register_with_payment_result = await registrar.register_with_payment({
            name: name,
            years: parseInt(years),
            approve_amount: amount
        });
    });
Then(/^Last register_with_payment result is '([^']*)'$/,
    function (status: string) {
        assert_remote_result(global_register_with_payment_result, status)
    });
