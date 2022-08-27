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

Then(/^registrar metadata "([^"]*)" result is$/, async function (name, table) {

    const token_id = (await registrar.get_token_details_by_names([name]))[0];
    logger.debug(`token_id: ${token_id}`)
    const result = await registrar.metadata(name)
    logger.debug(result)
    //assert_remote_result(result, table)
});
Then(/^registrar getTokens result is$/, async function (name, table) {
    const result = await registrar.getTokens()
    assert_remote_result(result, table)
});
Then(/^registrar getRegistry result is$/, function () {

});
Then(/^registrar supply result is "([^"]*)"$/, async function (count) {
    const result = await registrar.supply()
    assert_remote_result(result, count)
});
