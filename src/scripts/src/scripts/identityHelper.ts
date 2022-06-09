import { identity, IdentityInfo, canister } from '@deland-labs/ic-dev-kit'
import { Principal } from '@dfinity/principal'

const getIdentity = (user: string): IdentityInfo => {
    return identity.identityFactory.getIdentity(`icnaming_${user}`)!
}

const getPrincipal = (user: string): Principal => {
    let userPrincipal = getIdentity(user);
    if (userPrincipal) {
        return userPrincipal.identity.getPrincipal();
    }
    try {
        return Principal.fromText(user)
    } catch {
        return canister.get_principal(user)
    }
}

class Identities {
    public get main(): IdentityInfo {
        return getIdentity("main")
    }
    public get user1(): IdentityInfo {
        return getIdentity("user1")
    }
    getIdentity(user: string): IdentityInfo {
        return getIdentity(user)
    }
    getPrincipal(user: string): Principal {
        return getPrincipal(user)
    }
}

export const identities = new Identities()