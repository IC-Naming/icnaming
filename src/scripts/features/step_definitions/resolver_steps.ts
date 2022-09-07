import "./setup"
import {DataTable, Given, Then, When} from '@cucumber/cucumber'
import {createResolver, resolver} from '~/declarations/resolver'
import {
    BatchGetReverseResolvePrincipalResponse,
    BooleanActorResponse as EnsureResolverCreatedResult,
    BooleanActorResponse as UpdateRecordValueResult, ImportRecordValueRequest, ResolverValueImportItem
} from '~/declarations/resolver/resolver.did'
import {expect} from 'chai'
import {Result} from '~/utils/Result'
import {assert_remote_result} from './utils'
import {identities} from '~/identityHelper'
import {Principal} from "@dfinity/principal"
import logger from "node-color-log";
import {utils} from "@deland-labs/ic-dev-kit";

let global_ensure_resolver_created_result: EnsureResolverCreatedResult
let global_update_record_value_result: UpdateRecordValueResult

let global_batch_get_reverse_resolve_principal_result: BatchGetReverseResolvePrincipalResponse

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
                const target_row = rows.find(row => {

                    if (item[0] == "account_id.icp") {
                        logger.debug(`row:${utils.principalToAccountID(identities.getPrincipal(row.to))}, item:${item[1]}`);
                        return row.from == item[0] && utils.principalToAccountID(identities.getPrincipal(row.to)) == item[1];
                    }

                    logger.debug(`row:${identities.getPrincipal(row.to).toText()}, item:${item[1]}`);
                    return row.from == item[0] && identities.getPrincipal(row.to).toText() == item[1];
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
When(/^batch get reverse resolve principal$/, async function (table) {
    let dataTable = table.hashes()
    let principals: Principal[] = dataTable.map((item) => identities.getPrincipal(item.user))
    global_batch_get_reverse_resolve_principal_result = await resolver.batch_get_reverse_resolve_principal(principals)
});
Then(/^batch check reverse resolve principal$/, async function (table) {

    let dataTable = table.hashes()
    if ('Ok' in global_batch_get_reverse_resolve_principal_result) {
        let list = global_batch_get_reverse_resolve_principal_result.Ok.map((item) => {
            return {
                principal: item[0],
                name: item[1][0]
            }
        })
        for (let data of dataTable) {
            let target = list.find((item) => {
                return item.principal.toText() == identities.getPrincipal(data.user).toText()
            })
            if (data.name == 'undefined') {
                expect(target?.name).to.undefined
            } else {
                expect(target?.name).to.equal(data.name)
            }
        }

    } else {
        expect.fail(`batch check reverse resolve principal failed: ${global_batch_get_reverse_resolve_principal_result.Err}`)
    }
});
When(/^import_record_value$/, async function (table) {
    let dataTable: ResolverValueImportItem[] = table.hashes().map((item) => {

        let operation = item.operation;
        let value_and_operation;
        if (operation == 'InsertOrIgnore') {
            value_and_operation = {
                InsertOrIgnore: item.value
            }
        } else if (operation == 'Upsert') {
            value_and_operation = {
                Upsert: item.value
            }
        } else if (operation == 'Remove') {
            value_and_operation = {
                Remove: null
            }
        } else {
            expect.fail(`import_record_value failed: ${operation} not support`)
        }
        return {
            key: item.key,
            name: item.name,
            value_and_operation: value_and_operation
        } as ResolverValueImportItem
    })
    const identityInfo = identities.getIdentity("main")
    let localResolver = createResolver(identityInfo)
    let request = {
        items: dataTable
    } as ImportRecordValueRequest
    logger.debug(`import_record_value request: ${JSON.stringify(request)}`)
    let result = await localResolver.import_record_value(request)
    assert_remote_result(result, 'Ok')

});
Then(/^batch check record_value$/, async function (table) {
    let dataTable = table.hashes()
        .map((item) => {
            return {
                name: item.name,
                key: item.key,
                value: item.value
            }
        })
    for (let data of dataTable) {
        let result = await resolver.get_record_value(data.name)
        logger.debug(result);
        if ('Ok' in result) {
            let target = result.Ok.find((item) => {
                return item[0] == data.key
            })
            if (target) {
                expect(target[1]).to.equal(data.value)
            } else {
                expect.fail(`batch check record_value failed: ${data.name} ${data.key} not found`)
            }
        } else {
            expect.fail(`batch check record_value failed: ${result.Err}`)
        }
    }
});
Then(/^batch check record_value should not in$/, async function (table) {
    let dataTable = table.hashes()
        .map((item) => {
            return {
                name: item.name,
                key: item.key,
                value: item.value
            }
        })
    for (let data of dataTable) {
        let result = await resolver.get_record_value(data.name)
        logger.debug(result);
        if ('Ok' in result) {
            let target = result.Ok.find((item) => {
                return item[0] == data.key
            })
            expect(target).to.undefined
        } else {
            expect.fail(`batch check record_value failed: ${result.Err}`)
        }
    }
});
