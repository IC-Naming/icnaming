import {Principal} from "@dfinity/principal";
import fs from "fs";
import * as csvParser from "csv-parser";

export const isCanisterId = (value: string): boolean => {
    let _ = Principal.fromText(value)
    return value.length <= "fgzg4-siaaa-aaaam-aafpa-cai".length;
}

export class Registrar {
    public name: string;
    public owner: string;

    constructor(name: string, owner: string) {
        this.name = name;
        this.owner = owner;
        // try parse
        let _ = Principal.fromText(owner)
    }

    public isOwnedByMarket(): boolean {
        return isCanisterId(this.owner);
    }
}

export class Resolver {
    public name: string;
    public values: Map<string, string>;

    constructor(name: string, values: Map<string, string>) {
        this.name = name;
        this.values = values;
    }
}

export class ResolverReverseName {
    public name: string;
    public value: string;

    constructor(name: string, value: string) {
        this.name = name;
        this.value = value;
        // try parse
        let _ = Principal.fromText(value)
    }
}

class CsvLoader<T> {
    public filePath: string;

    constructor(filePath: string) {
        this.filePath = filePath;
    }

    public async Load(): Promise<T[]> {
        const items: T[] = []
        let job = new Promise<void>((resolve) => {
            fs.createReadStream(this.filePath)
                .pipe(csvParser.default())
                .on('data', (data) => {
                    items.push(data)
                })
                .on('end', resolve)
        })
        await job
        return items
    }
}

interface RegistrarRecordItem {
    name: string;
    owner: string;
}

export class RegistrarCsvLoader {
    public filePath: string;
    public registrars: Map<string, Registrar>;
    public userNames: Map<string, Registrar[]>;

    constructor(filePath: string) {
        this.filePath = filePath;
    }

    public async Load(): Promise<void> {
        let csvLoader = new CsvLoader<RegistrarRecordItem>(this.filePath);
        let items = await csvLoader.Load();
        let registrars = new Map<string, Registrar>();
        let userNames = new Map<string, Registrar[]>();
        for (let item of items) {
            registrars.set(item.name, new Registrar(item.name, item.owner));
            let userNameItems = userNames.get(item.owner);
            if (!userNameItems) {
                userNameItems = [];
                userNames.set(item.owner, userNameItems);
            }
            userNameItems.push(new Registrar(item.name, item.owner));
        }
        this.registrars = registrars;
        this.userNames = userNames;
    }
}

interface ResolverRecordItem {
    name: string;
    key: string;
    value: string;
}

export class ResolverCsvLoader {
    public filePath: string;
    public resolvers: Map<string, Resolver>;

    constructor(filePath: string) {
        this.filePath = filePath;
    }

    public async Load(): Promise<void> {
        let csvLoader = new CsvLoader<ResolverRecordItem>(this.filePath);
        let items = await csvLoader.Load();
        let resolvers = new Map<string, Resolver>();
        for (let item of items) {
            if (resolvers.has(item.name)) {
                let resolver = resolvers.get(item.name);
                resolver!.values.set(item.key, item.value);
            } else {
                let resolver = new Resolver(item.name, new Map<string, string>());
                resolver.values.set(item.key, item.value);
                resolvers.set(item.name, resolver);
            }
        }
        this.resolvers = resolvers;
    }
}

interface ResolverReverseRecordItem {
    name: string;
    value: string;
}

export class ResolverReverseNameCsvLoader {
    public filePath: string;
    public resolverReverseNames: Map<string, ResolverReverseName>;

    constructor(filePath: string) {
        this.filePath = filePath;
    }

    public async Load(): Promise<void> {
        let csvLoader = new CsvLoader<ResolverReverseRecordItem>(this.filePath);
        let items = await csvLoader.Load();
        let re = new Map<string, ResolverReverseName>();
        for (let item of items) {
            re.set(item.name, new ResolverReverseName(item.name, item.value));
        }
        this.resolverReverseNames = re;
    }
}