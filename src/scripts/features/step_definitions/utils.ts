import {dicp} from '../../src/scripts/declarations/dicp'
import {Given, Then, When} from '@cucumber/cucumber'
import {reinstall_all} from '../../src/tasks'
import {expect} from 'chai'
import {canister, utils} from '@deland-labs/ic-dev-kit'
import logger from 'node-color-log'
import {identities} from '~/identityHelper'

Then(/^Sleep for "([^"]*)" secs.$/, async function (sec: string) {
    // sleep for secs
    await new Promise(resolve => setTimeout(resolve, parseFloat(sec) * 1000))
})

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
