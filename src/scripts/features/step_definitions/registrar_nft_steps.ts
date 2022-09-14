import "./setup"
import {Given, Then, When} from '@cucumber/cucumber'
import {createRegistrar, registrar} from '~/declarations/registrar'

import {expect} from 'chai'

import {
    AllowanceActorResponse,
    AllowanceRequest,
    ApproveRequest,
    EXTTransferResponse,
    ImportTokenIdResponse,
    TransferRequest,
    User,
} from '~/declarations/registrar/registrar.did'
import {identities} from '~/identityHelper'

import {assert_remote_result} from './utils'
import logger from 'node-color-log'
import {IDL} from '@dfinity/candid'
import {utils} from "@deland-labs/ic-dev-kit";


interface getTokensDto {
    index: number,
    key: string,
    value: string,
}

let global_transfer_result_list: EXTTransferResponse[] = []
let global_allowance_result_list: AllowanceActorResponse[] = []
let global_import_token_id_from_registration_result: ImportTokenIdResponse

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
    expect(token_id).to.not.be.undefined
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
    const dataTable: getTokensDto[] = table.hashes()
        .map((item) => {
            return {
                index: Number(item.index),
                key: item.key,
                value: item.value,
            }
        })
    for (let i = 0; i < dataTable.length; i++) {
        let targetData = dataTable[i]
        let index = result[i][0]
        let nonfungible = result[i][1]
        if ('nonfungible' in nonfungible && nonfungible.nonfungible.metadata.length) {
            let metadata = nonfungible.nonfungible.metadata[0].map((item) => {
                return Number(item)
            })
            let args = IDL.decode(get_metadata_type(), Buffer.from(metadata))
            let exp = args.map((item) => {
                return {
                    index: index,
                    key: item[0][0],
                    value: item[0][1]
                } as getTokensDto
            })[0];
            logger.debug(`args: ${JSON.stringify(args)}`)
            expect(index).to.equal(targetData.index)
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
            const principal = identities.getPrincipal(targetData.user)
            const accountId = utils.principalToAccountID(principal)
            if ('Ok' in result) {
                logger.debug(`result: ${JSON.stringify(result)}`)
                expect(result.Ok).to.equal(accountId)
            }
        } else {
            expect(false, 'get registrar bearer failed').to.equal(true)
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

            let from: User
            if (targetData.from_type == "principal") {
                from = {
                    principal: identities.getPrincipal(targetData.from)
                } as User
            } else {
                from = {
                    address: targetData.from
                } as User
            }
            let to: User
            if (targetData.to_type == "principal") {
                to = {
                    principal: identities.getPrincipal(targetData.to)
                } as User
            } else {
                to = {
                    address: targetData.to
                } as User
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
Given(/^registrar ext_approve name to spender, the caller is the name owner$/, async function (table) {
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
            await registrar.ext_approve(approve_request)
        }
    }
});
When(/^last registrar ext_transfer result is err, expected err is "([^"]*)" and message is "([^"]*)"$/, function (err, message) {
    let last_result = global_transfer_result_list[global_transfer_result_list.length - 1]
    if ('Err' in last_result) {
        if (err in last_result.Err) {
            expect(last_result.Err[err]).to.equal(message)
        }
    }
});
Given(/^registrar allowance action, caller is none/, async function (table) {
    let dataTable = table.hashes()
    for (let targetData of dataTable) {
        const spender = identities.getPrincipal(targetData.to)
        let owner
        if (targetData.from_type == 'principal') {
            owner = {
                principal: identities.getPrincipal(targetData.from)
            } as User
        } else {
            owner = {
                address: targetData.from
            } as User
        }
        const token = await get_token_id_by_name(targetData.name)
        if (token != undefined) {
            let request = {
                token: token,
                owner: owner,
                spender: spender
            } as AllowanceRequest
            const result = await registrar.allowance(request)
            global_allowance_result_list.push(result)
        }
    }
});
When(/^all registrar allowance is ok$/, function () {
    for (let result of global_allowance_result_list) {
        assert_remote_result(result)
    }
});
When(/^last registrar allowance result is err, expected err is "([^"]*)" and message is "([^"]*)"$/, function (err, message) {
    let last_result = global_allowance_result_list[global_allowance_result_list.length - 1]
    if ('Err' in last_result) {
        if (err in last_result.Err) {
            expect(last_result.Err[err]).to.equal(message)
        }
    }
});
When(/^all registrar allowance is ok, and the value is "([^"]*)"$/, function (value) {
    for (let result of global_allowance_result_list) {
        assert_remote_result(result)
        if ('Ok' in result) {
            expect(result.Ok).to.equal(BigInt(value))
        }
    }
});
Given(/^registrar import token id from registration$/, async function () {
    global_import_token_id_from_registration_result = await registrar.import_token_id_from_registration()
});
When(/^last registrar import token id from registration result is ok, and value is "([^"]*)"$/, function (value) {
    assert_remote_result(global_import_token_id_from_registration_result)
    if ('Ok' in global_import_token_id_from_registration_result) {
        expect(global_import_token_id_from_registration_result.Ok).to.equal(BigInt(value))
    }
});
