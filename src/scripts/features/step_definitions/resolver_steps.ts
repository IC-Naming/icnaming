import "./setup"
import { Given, Then, When } from '@cucumber/cucumber'
import { createResolver, resolver } from '~/declarations/resolver'
import {
  BooleanActorResponse as EnsureResolverCreatedResult,
  BooleanActorResponse as UpdateRecordValueResult
} from '~/declarations/resolver/resolver.did'
import { expect } from 'chai'
import { Result } from '~/utils/Result'
import { assert_remote_result } from './utils'
import { identities } from '~/identityHelper'

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
  async function (name: string, data) {
    const results = await new Result(resolver.get_record_value(name)).unwrap()
    const rows = data.rows()
    if (rows.length == 0) {
      expect(results.length).to.equal(0)
    } else {
      expect(results.length).to.equal(rows.length)
      for (const item of results) {
        const target_row = rows.find(row => {
          return row[0] = item[0]
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
