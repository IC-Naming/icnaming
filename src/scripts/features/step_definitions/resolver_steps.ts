import "./setup"
import {DataTable, Given, Then, When} from '@cucumber/cucumber'
import {createResolver, resolver} from '~/declarations/resolver'
import {
    BooleanActorResponse as EnsureResolverCreatedResult,
    BooleanActorResponse as UpdateRecordValueResult
} from '~/declarations/resolver/resolver.did'
import {expect} from 'chai'
import {Result} from '~/utils/Result'
import {assert_remote_result} from './utils'
import {identities} from '~/identityHelper'
import {Principal} from "@dfinity/principal"
import logger from "node-color-log";
import {identity, utils} from "@deland-labs/ic-dev-kit";

let global_ensure_resolver_created_result: EnsureResolverCreatedResult
let global_update_record_value_result: UpdateRecordValueResult

When(/^I call ensure_resolver_created "([^"]*)"$/,
    async function (name: string) {
        global_ensure_resolver_created_result = await resolver.ensure_resolver_created(name)
    })
Then(/^ensure_resolver_created result in status "([^"]*)"$/,
    function (
        status: string) {
        assert_remote_result(global_ensure_resolver_created_result, status)
    })
Then(/^get_record_value "([^"]*)" should be as below$/,
    async function (name: string, data: DataTable) {
        const results = await new Result(resolver.get_record_value(name)).unwrap()
        const rows = data.rows()
        if (rows.length == 0) {
            expect(results.length).to.equal(0, "expected no results")
        } else {
            expect(results.length).to.equal(rows.length, "expected same number of results")
            for (const item of results) {
                const target_row = rows.find(row => {
                    return row[0] = item[0]
                })
                expect(target_row).to.not.equal(undefined)
            }
        }
    })
Then(/^auto resolve get_record_value "([^"]*)" should be as below$/,
    async function (name: string, data: DataTable) {
        const results = await new Result(resolver.get_record_value(name)).unwrap()
        logger.debug(results);
        const rows = data.hashes()
        logger.debug(rows);
        if (rows.length == 0) {
            expect(results.length).to.equal(0, "expected no results")
        } else {
            expect(results.length).to.equal(rows.length, "expected same number of results")
            for (const item of results) {
                const target_row = rows.find(row =>{

                    if (item[0] == "account_id.icp") {
                        logger.debug(`row:${utils.principalToAccountID(identities.getPrincipal(row.to))}, item:${item[1]}`);
                        return  row.from == item[0] && utils.principalToAccountID(identities.getPrincipal(row.to)) == item[1];
                    }

                    logger.debug(`row:${identities.getPrincipal(row.to).toText()}, item:${item[1]}`);
                    return  row.from == item[0] && identities.getPrincipal(row.to).toText() == item[1];
                })
                expect(target_row).to.not.equal(undefined)
            }
        }
    })

async function update_resolver(resolver: any, data, name: string) {
    const rows = data.rows()
    global_update_record_value_result = await resolver.set_record_value(name, rows)
}

Given(/^User "([^"]*)" update resolver "([^"]*)" with values$/,
    async function (user: string, name: string, data) {
        const identityInfo = identities.getIdentity(user)
        const resolver = createResolver(identityInfo)
        await update_resolver(resolver, data, name)
    })

When(/^I update resolver "([^"]*)" with values$/,
    async function (name: string, data) {
        await update_resolver(resolver, data, name)
    })

Then(/^update_record_value result in status '([^']*)'$/,
    function (status: string) {
        assert_remote_result(global_update_record_value_result, status)
    })
Then(/^Reverse resolve name "([^"]*)" should be "([^"]*)"$/,
    async function (principal: string, name: string) {
        let result = await resolver.reverse_resolve_principal(Principal.fromText(principal));
        if ('Err' in result) {
            expect.fail(`Reverse resolve name ${principal} failed: ${result.Err}`)
        } else {
            if (name === "none") {
                expect(result.Ok.length).to.equal(0);
            } else {
                let item = result.Ok[0];
                expect(item).to.equal(name);
            }
        }
    })
When(/^I update resolver "([^"]*)" with "([^"]*)" keys$/,
    async function (name: string, keysCount: string) {
        let items: [string, string][] = [];
        for (let i = 0; i < parseInt(keysCount); i++) {
            items.push([`key${i}`, `value${i}`]);
        }
        global_update_record_value_result = await resolver.set_record_value(name, items)
    })