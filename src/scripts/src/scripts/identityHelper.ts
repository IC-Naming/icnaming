import {identity, IdentityInfo, canister} from '@deland-labs/ic-dev-kit'
import {Principal} from '@dfinity/principal'

const getIdentity = (user: string): IdentityInfo => {
    return identity.identityFactory.getIdentity(`icnaming_${user}`)!
}

const getPrincipal = (user: string): Principal => {
    if (user == "anonymous") {
        return Principal.anonymous();
    }
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

    public get user2(): IdentityInfo {
        return getIdentity("user2")
    }

    public get user3(): IdentityInfo {
        return getIdentity("user3")
    }

    public get allUsers(): [string,IdentityInfo][] {
        return [["main",this.main],["user1",this.user1],["user2",this.user2],["user3",this.user3]]
    }

    getIdentity(user: string): IdentityInfo {
        return getIdentity(user)
    }

    getPrincipal(user: string): Principal {
        return getPrincipal(user)
    }

    getUserByPrincipal(principal: string): string |undefined {

        for (let user of this.allUsers) {
            if (user[1].identity.getPrincipal().toText() == principal) {
                return user[0]
            }
        }
        return undefined

    }
}

export const identities = new Identities()
