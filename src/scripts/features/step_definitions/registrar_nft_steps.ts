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


async function getRegistry() {
    const result = await registrar.getRegistry();
    return result;
}

async function getToken(name: string) {
    const data = await getRegistry();
    const result = data.find(item => item[1] == name);
    return result;
}


Then(/^registrar metadata "([^"]*)"$/, async function (name: string) {

    const token = await getToken(name);
    const tokenId = token[0];
});
