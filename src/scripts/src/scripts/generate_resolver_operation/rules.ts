import {isCanisterId, Registrar, Resolver, ResolverReverseName} from "~/generate_resolver_operation/loaders";
import {utils} from "@deland-labs/ic-dev-kit";
import {Principal} from "@dfinity/principal";

export enum ResolverKeys {
    resolverReverseKey = "settings.reverse_resolution.principal",
    accountIdKey = "account_id.icp",
    principalKey = "principal.icp"
}

export class ResolverWithoutRegistrarRule {
    public Check(registrars: Map<string, Registrar>, resolvers: Map<string, Resolver>): Resolver[] {
        let re: Resolver[] = [];
        for (let resolver of resolvers.values()) {
            if (!registrars.has(resolver.name)) {
                re.push(resolver)
            }
        }
        return re;
    }
}


export interface MissingResolverValueItem {
    registrar: Registrar,
    resolver: Resolver | undefined,
    key: ResolverKeys,
    value: string,
}

export class RegistrarWithoutFullResolver {
    public Check(registrars: Map<string, Registrar>,
                 resolvers: Map<string, Resolver>): MissingResolverValueItem[] {
        let re: MissingResolverValueItem[] = [];
        const checkingKeys = [ResolverKeys.accountIdKey, ResolverKeys.principalKey];
        for (let registrar of registrars.values()) {
            if (!registrar.isOwnedByMarket()) {
                let resolver = resolvers.get(registrar.name);
                if (!resolver) {
                    checkingKeys.forEach(key => {
                        re.push({
                            registrar: registrar,
                            resolver: resolver!,
                            key: key,
                            value: this.createValue(registrar, key)
                        })
                    })
                } else {
                    checkingKeys.forEach(key => {
                        if (!resolver!.values.has(key)) {
                            re.push({
                                registrar: registrar,
                                resolver: resolver!,
                                key: key,
                                value: this.createValue(registrar, key)
                            })
                        }
                    })
                }
            }
        }
        return re;
    }

    private createValue(registrar: Registrar, key: ResolverKeys): string {
        switch (key) {
            case ResolverKeys.accountIdKey:
                return utils.principalToAccountID(Principal.fromText(registrar.owner));
            case ResolverKeys.principalKey:
                return registrar.owner;
            default:
                throw new Error("Not supported key: " + key);
        }
    }
}


export interface CreateDefaultReverseNameItem {
    defaultReverseName: Registrar,
}

export class CreateDefaultResolverReverseNameRule {
    public Check(userNames: Map<string, Registrar[]>,
                 resolverReverseNames: Map<string, ResolverReverseName>): CreateDefaultReverseNameItem[] {
        let re: CreateDefaultReverseNameItem[] = [];
        for (let [ownerPrincipal, names] of userNames.entries()) {
            if (!isCanisterId(ownerPrincipal)) {
                let userHasReverseName = false;
                for (const registrar of names) {
                    if (resolverReverseNames.has(registrar.name)) {
                        userHasReverseName = true;
                        break;
                    }
                }
                if (!userHasReverseName) {
                    // sort names by name asc
                    names.sort((a, b) => {
                        if (a.name < b.name) {
                            return -1;
                        }
                        if (a.name > b.name) {
                            return 1;
                        }
                        return 0;
                    });
                    re.push({
                        defaultReverseName: names[0]
                    })
                }
            }
        }
        return re;
    }
}

interface MismatchResolverReverseNameItem {
    resolverReverseName: ResolverReverseName,
}

export class ResolverReverseNameMatchingRule {
    public Check(registrars: Map<string, Registrar>,
                 resolverReverseNames: Map<string, ResolverReverseName>): MismatchResolverReverseNameItem[] {
        let re: MismatchResolverReverseNameItem[] = [];
        for (let [name, resolverReverseName] of resolverReverseNames.entries()) {
            let registrar = registrars.get(name);
            if (!registrar) {
                re.push({
                    resolverReverseName
                })
            } else if (registrar.owner != resolverReverseName.value) {
                re.push({
                    resolverReverseName
                })
            }
        }
        return re;
    }
}
