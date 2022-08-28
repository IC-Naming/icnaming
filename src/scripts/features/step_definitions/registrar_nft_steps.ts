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
    GetNameStatueActorResponse, User, TransferRequest, EXTTransferResponse, ApproveRequest,
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
import {IDL} from '@dfinity/candid'


let global_transfer_result_list: EXTTransferResponse[] = []

export const get_metadata_type = () => {
    return [IDL.Vec(IDL.Tuple(
        IDL.Text,
        IDL.Text,
    ))]
}

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

const get_transfer_request = (from: User, to: User, token: string) => {
    return {
        from: from,
        to: to,
        token: token,
        amount: BigInt(1),
        subaccount: [],
        notify: false,
        memo: [],
    } as TransferRequest
}

const get_name_bear = async (name: string) => {
    const token_id = await get_token_id_by_name(name)
    if (token_id != undefined) {
        const result = await registrar.bearer(token_id)
        if ('Ok' in result) {
            return identities.getUserByPrincipal(result.Ok)
        }
    }
}


Then(/^registrar metadata "([^"]*)" result is$/, async function (name, table) {

    let dataTable = table.hashes()
    let token_id = await get_token_id_by_name(name)
    if (token_id != undefined) {
        logger.debug(`id: ${JSON.stringify(token_id)}`)
        const result = await registrar.metadata(token_id)
        assert_remote_result(result)
        if ('Ok' in result && 'nonfungible' in result.Ok) {
            const metadata = result.Ok.nonfungible.metadata[0]
            if (metadata != undefined) {
                let targetData = dataTable[0];
                let args = IDL.decode(get_metadata_type(), Buffer.from(metadata))
                let exp = args.map((item) => {
                    return {
                        'key': item[0][0],
                        'value': item[0][1]
                    }
                })[0];
                expect(exp.key).to.equal(targetData.key)
                expect(exp.value).to.equal(targetData.value)
                return;
            }
        }
    }
    expect(false, 'get registrar metadata failed').to.equal(true)
});
Then(/^registrar getTokens result is$/, async function (table) {
    const result = await registrar.getTokens()
    const dataTable = table.hashes()
    for (let i = 0; i < dataTable.length; i++) {
        let targetData = dataTable[i]
        let index = result[i][0]
        let nonfungible = result[i][1]
        if ('nonfungible' in nonfungible) {
            let metadata = nonfungible.nonfungible.metadata[0].map((item) => {
                return Number(item)
            })
            let args = IDL.decode(get_metadata_type(), Buffer.from(metadata))
            let exp = args.map((item) => {
                return {
                    'key': item[0][0],
                    'value': item[0][1]
                }
            })[0];
            logger.debug(`args: ${JSON.stringify(args)}`)
            expect(index).to.equal(Number(targetData.index))
            expect(exp.key).to.equal(targetData.key)
            expect(exp.value).to.equal(targetData.value)
        } else {
            expect(false, 'get registrar getTokens failed').to.equal(true)
        }
    }


});
Then(/^registrar getRegistry result is$/, async function (table) {
    const result = await registrar.getRegistry()
    let dataTable = table.hashes()
    for (let i = 0; i < dataTable.length; i++) {
        let targetData = dataTable[i]
        expect(result[i][0]).to.equal(Number(targetData.index))
        expect(result[i][1]).to.equal(targetData.name)
    }
});
Then(/^registrar supply result is "([^"]*)"$/, async function (count) {
    const result = await registrar.supply()
    if ('Ok' in result) {
        expect(Number(result.Ok)).to.equal(Number(count))
    }
});
Then(/^registrar bearer result is$/, async function (table) {

    let dataTable = table.hashes()

    for (let targetData of dataTable) {
        const id = await get_token_id_by_name(targetData.name)
        if (id != undefined) {
            const result = await registrar.bearer(id)
            const principal = identities.getPrincipal(targetData.user).toText()
            if ('Ok' in result) {
                logger.debug(`result: ${JSON.stringify(result)}`)
                expect(result.Ok).to.equal(principal)
            }
        }
    }
});


Given(/^registrar ext_transfer action$/, async function (table) {
    let dataTable = table.hashes()
    for (let targetData of dataTable) {
        const id = await get_token_id_by_name(targetData.name)

        const caller = targetData.caller
        let localRegistrar
        if (targetData.caller != 'none') {
            const identityInfo = identities.getIdentity(caller)
            localRegistrar = createRegistrar(identityInfo)
        } else {
            localRegistrar = registrar
        }
        if (id != undefined) {
            let from: User = {
                principal: identities.getPrincipal(targetData.from)
            }
            let to: User = {
                principal: identities.getPrincipal(targetData.to)
            }

            const result = await localRegistrar
                .ext_transfer(get_transfer_request(from, to, id))
            global_transfer_result_list.push(result)
        }
    }

});
When(/^all registrar ext_transfer is ok$/, function () {
    for (let result of global_transfer_result_list) {
        assert_remote_result(result)
    }
});
Given(/^registrar ext_approve action$/, async function (table) {
    let dataTable = table.hashes()
    for (let targetData of dataTable) {
        const spender = targetData.spender

        const owner = await get_name_bear(targetData.name)
        const token = await get_token_id_by_name(targetData.name)
        logger.debug(`owner: ${JSON.stringify(owner)}`)
        if (owner != undefined) {
            const identityInfo = identities.getIdentity(owner)
            let registrar = createRegistrar(identityInfo)
            let approve_request = {
                token: token,
                subaccount: [],
                allowance: BigInt(1),
                spender: identities.getPrincipal(spender)
            } as ApproveRequest
            const result = await registrar.ext_approve(approve_request)

        }
    }
});
