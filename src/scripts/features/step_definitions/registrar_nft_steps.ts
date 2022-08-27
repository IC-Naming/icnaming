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


const get_token_id_by_name = async (name: string) => {
    const token_ids = await registrar.get_token_details_by_names([name])

    let map = token_ids.map((item) => {
        return {
            [item[0]]: {
                tokenId: item[1][0],
            }
        }
    })
    let token_id = map[0][name].tokenId;

    if (token_id != undefined) {
        return token_id[1]
    }


}

Then(/^registrar metadata "([^"]*)" result is$/, async function (name, table) {
    let token_id = await get_token_id_by_name(name)
    if (token_id != undefined) {
        logger.debug(`id: ${JSON.stringify(token_id)}`)
        const result = await registrar.metadata(token_id)
        assert_remote_result(result)
        if ('Ok' in result && 'nonfungible' in result.Ok) {
            const metadata = result.Ok.nonfungible.metadata[0]
            if (metadata != undefined) {
                //metadata byte to string
                const metadata_str = Buffer.from(metadata).toString('utf8')
                //const metadata_json = JSON.parse(metadata_str)
                logger.debug(`metadata: ${metadata_str}`)
            }
        }
    }
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
