import "./setup"
import {Then, When} from '@cucumber/cucumber'
import {createRegistry, registry} from '~/declarations/registry'
import {Result} from '~/utils/Result'
import {canister} from '@deland-labs/ic-dev-kit'
import {resolver} from '~/canisters/names'
import {Principal} from '@dfinity/principal'
import {expect} from 'chai'
import {identities} from '~/identityHelper'
import {GetDetailsResponse, RegistryDto} from '~/declarations/registry/registry.did'
import logger from 'node-color-log'

let global_set_subdomain_owner_result: GetDetailsResponse

When(/^I call set_subdomain_owner to add a second level name$/,
    async function () {
        const call = registry.set_subdomain_owner('hello2.ic',
            'ic',
            identities.user1.identity.getPrincipal(),
            BigInt(600),
            Principal.fromText(canister.get_id(resolver)))
        global_set_subdomain_owner_result = await call
    })
Then(/^set_subdomain_owner result in status "([^"]*)"$/,
    async function (status: string) {
        if (status === 'Ok') {
            expect('Ok' in global_set_subdomain_owner_result).to.be.true
        } else {
            if ('Err' in global_set_subdomain_owner_result) {
                expect(global_set_subdomain_owner_result.Err.message).to.equal(status)
            } else {
                expect.fail('Expected Err but got Ok')
            }
        }
    })
Then(/^get_resolver "([^"]*)" should be the public resolver$/,
    async function (name: string) {
        const resolver_value = await new Result(registry.get_resolver(name)).unwrap()
        const public_resolver = canister.get_id(resolver)
        expect(resolver_value.toText()).to.equal(public_resolver)
    })
Then(/^get_owner "([^"]*)" should be "([^"]*)"$/,
    async function (name: string, owner: string) {
        const owner_value = await new Result(registry.get_owner(name)).unwrap()
        const owner_principal = identities.getIdentity(owner).principalText
        expect(owner_value.toText()).to.equal(owner_principal)
    })

Then(/^get_ttl "([^"]*)" should be "([^"]*)"$/,
    async function (name: string, ttl: string) {
        const ttl_value = await new Result(registry.get_ttl(name)).unwrap()
        expect(ttl_value).to.equal(BigInt(ttl))
    })
Then(/^registry get_details "([^"]*)" should be as below$/,
    async function (name: string, data) {
        const details: RegistryDto = await new Result(registry.get_details(name)).unwrap()
        const expected = data.rowsHash()
        expect(details.owner.toText()).to.equal(identities.getIdentity(expected.owner).principalText)
        expect(details.resolver.toText()).to.equal(expected.resolver === 'public' ? canister.get_id(resolver) : expected.resolver)
        expect(details.ttl).to.equal(BigInt(expected.ttl))
        expect(details.name).to.equal(name)
    })
When(/^I update registry "([^"]*)" with values$/,
    async function (name: string, data) {
        const input: { ttl, resolver } = data.rowsHash()
        const registry = createRegistry(identities.main)
        await new Result(registry.set_record(name, BigInt(input.ttl), Principal.fromText(input.resolver))).unwrap()
    })
When(/^User "([^"]*)" set registry owner for "([^"]*)" to "([^"]*)"$/,
    async function (user: string, name: string, new_owner: string) {
        const registry = createRegistry(identities.getIdentity(user))
        await registry.set_owner(name, identities.getPrincipal(new_owner))
    })
When(/^I update registry "([^"]*)" resolver to "([^"]*)"$/,
    async function (name: string, new_resolver: string) {
        const registry = createRegistry(identities.main)
        const result = await registry.set_resolver(name, Principal.fromText(new_resolver))
        logger.debug(`set_resolver result: ${result.toString()}`)
    })
