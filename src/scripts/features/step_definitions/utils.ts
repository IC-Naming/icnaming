import {dicp} from '../../src/scripts/declarations/dicp'
import {Given, Then, When} from '@cucumber/cucumber'
import {reinstall_all} from '../../src/tasks'
import {expect} from 'chai'
import {canister, utils} from '@deland-labs/ic-dev-kit'
import logger from 'node-color-log'
import {identities} from '~/identityHelper'
import fs from "fs";
import * as csv from "csv-parser";
import {resolver} from '~/declarations/resolver'

Then(/^Sleep for "([^"]*)" secs.$/, async function (sec: string) {
    // sleep for secs
    await new Promise(resolve => setTimeout(resolve, parseFloat(sec) * 1000))
})


export const import_record_value_from_csv = async (file_name: string) => {
    const items: {
        name: string,
        operation: string,
        key: string,
        value: string
    }[] = []
    let job = new Promise<void>(resolve => {
        fs.createReadStream(`./scripts/features/data/${file_name}.csv`)
            .pipe(csv.default(
                {
                    headers: ['name', 'operation', 'key', 'value'],
                    skipLines: 1
                }
            ))
            .on('data', (data) => items.push(data))
            .on('end', () => {
                resolve();
            })
    })
    await job
    const result = await resolver.import_record_value({
        items: items.map(item => {
            let value_and_operation
            if (item.operation == 'InsertOrIgnore') {
                value_and_operation = {
                    InsertOrIgnore: item.value
                }
            } else if (item.operation == 'Upsert') {
                value_and_operation = {
                    Upsert: item.value
                }
            } else if (item.operation == 'Remove') {
                value_and_operation = {
                    Remove: null
                }
            } else {
                expect.fail(`import_record_value failed: ${item.operation} not support`)
            }
            return {
                name: item.name,
                key: item.key,
                value_and_operation: value_and_operation,
            }
        })
    })
    return result

}

export const set_balance_to = async (to: string, balance: string | BigInt): Promise<void> => {
    let balance_bigint
    if (typeof balance === 'string') {
        balance_bigint = utils.toICPe8s(balance)
    } else {
        balance_bigint = balance
    }
    const sub_account = []
    const amount = balance_bigint
    const created_at = []
    const result = await dicp.transfer(sub_account as [],
        to,
        amount,
        created_at as []
    )
    logger.debug(`Transfer result: ${JSON.stringify(result)}`)
}

export const reinstall_canisters = async (names: string[]): Promise<void> => {
    const canisters = {}
    for (const name of names) {
        canisters[name] = true
    }

    console.info(`Reinstalling canisters: ${JSON.stringify(canisters)}`)

    await reinstall_all({
        build: false,
        init: true,
        canisters
    })
}

export const assert_remote_result = (result: any, status?: string) => {
    if (!status || status === 'Ok') {
        expect('Ok' in result).to.be.equal(true, `Expected Ok but got ${JSON.stringify(result)}`)
    } else {
        if ('Err' in result) {
            expect(result.Err.message).to.be.equal(status)
        } else {
            expect.fail(`Expected to be error but found ${JSON.stringify(result)}`)
        }
    }
}

Given(/^Reinstall canisters$/,
    async function (data) {
        const target_canisters = data.hashes()
        const names: string[] = []
        for (const item of target_canisters) {
            names.push(item.name)
        }
        await reinstall_canisters(names)
    })
When(/^canister "([^"]*)" is down$/, async function (canister_name: string) {
    await canister.uninstall_code(canister_name)
})

Given(/^User "([^"]*)" balance is set to be "([^"]*)"$/,
    async function (user: string, balance: string) {
        await set_balance_to(identities.getPrincipal(user).toText(), balance)
    })
Then(/^User "([^"]*)" balance is "([^"]*)"$/,
    async function (user: string, balance_str: string) {
        const balance = utils.toICPe8s(balance_str)
        const balance_result = await dicp.balanceOf(identities.getPrincipal(user).toText())
        logger.debug(`Balance result: ${JSON.stringify(balance_result)}`)
        expect(balance_result).to.be.equal(balance)
    })
