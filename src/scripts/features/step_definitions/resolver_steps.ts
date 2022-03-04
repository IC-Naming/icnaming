import "~/setup";
import {Then, When} from "@cucumber/cucumber";
import {resolver} from "~/declarations/resolver";
import {
    BooleanActorResponse as EnsureResolverCreatedResult,
    BooleanActorResponse as UpdateRecordValueResult
} from "~/declarations/resolver/resolver.did";
import {expect} from "chai";
import {Result} from "~/utils/Result";
import {assert_remote_result} from "./utils";

let global_ensure_resolver_created_result: EnsureResolverCreatedResult;
let global_update_record_value_result: UpdateRecordValueResult;

When(/^I call ensure_resolver_created "([^"]*)"$/,
    async function (name: string) {
        global_ensure_resolver_created_result = await resolver.ensure_resolver_created(name);
    });
Then(/^ensure_resolver_created result in status "([^"]*)"$/,
    function (
        status: string) {
        assert_remote_result(global_ensure_resolver_created_result, status);
    });
Then(/^get_record_value "([^"]*)" should be as below$/,
    async function (name: string, data) {
        let results = await new Result(resolver.get_record_value(name)).unwrap();
        let rows = data.rows();
        if (rows.length == 0) {
            expect(results.length).to.equal(0);
        } else {
            expect(results.length).to.equal(rows.length);
            for (const item of results) {
                let target_row = rows.find(row => {
                    return row[0] = item[0];
                });
                expect(target_row).to.not.equal(undefined);
            }
        }

    });
When(/^I update resolver "([^"]*)" with values$/,
    async function (name: string, data) {
        let rows = data.rows();
        global_update_record_value_result = await resolver.set_record_value(name, rows);
    });

Then(/^update_record_value result in status '([^']*)'$/,
    function (status: string) {
        assert_remote_result(global_update_record_value_result, status);
    });