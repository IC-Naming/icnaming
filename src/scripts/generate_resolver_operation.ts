import {Principal} from "@dfinity/principal";
import logger from "node-color-log";
import {utils} from "@deland-labs/ic-dev-kit";
import fs from "fs";
import * as csvParser from "csv-parser";
import {createObjectCsvWriter} from "csv-writer";


interface RegistrarRecord {
    name: string,
    owner: string,
}

interface ResolverRecord {
    name: string,
    pairs: PairRecord[],
}

interface PairRecord {
    key: string,
    value: string
}

interface ResolverReverseRecord {
    name: string,
    value: string,
}

interface NameKeyRecord {
    name: string,
    key: string,
}

interface OperationRecord {
    name: string,
    key: string,
    value: string,
    operation: string
}

const resolverReverseKey = "settings.reverse_resolution.principal"
const accountIdKey = "account_id.icp"
const principalKey = "principal.icp"

const readRegistrarCsv = async () => {
    const items: RegistrarRecord[] = []
    let job = new Promise<void>((resolve, reject) => {
        fs.createReadStream('./scripts/features/data/' + 'RegistrarRecords.csv')
            .pipe(csvParser.default())
            .on('data', (data) => {
                items.push(data)
            })
            .on('end', resolve)
    })
    await job
    return items
}

const groupBy = <T, K extends keyof any>(arr: T[], key: (i: T) => K) =>
    arr.reduce((groups, item) => {
        (groups[key(item)] ||= []).push(item);
        return groups;
    }, {} as Record<K, T[]>);
const readResolverCsv = async () => {
    const items: {
        name: string,
        key,
        value,
    }[] = []
    let job = new Promise<void>((resolve, reject) => {
        fs.createReadStream('./scripts/features/data/' + 'ResolverRecords.csv')
            .pipe(csvParser.default())
            .on('data', (data) => items.push(data))
            .on('end', resolve)
    })
    await job
    let groups = groupBy(items, (item) => item.name)
    const resolverRecords: ResolverRecord[] = []
    for (const [key, value] of Object.entries(groups)) {
        const pairs: PairRecord[] = value.map((item) => {
            return {
                key: item.key,
                value: item.value
            }
        })
        resolverRecords.push({
            name: key,
            pairs: pairs
        })
    }
    return resolverRecords
}


const readResolverReverseCsv = async () => {
    const items: ResolverReverseRecord[] = []
    let job = new Promise<void>((resolve, reject) => {
        fs.createReadStream('./scripts/features/data/' + 'ResolverReverseRecords.csv')
            .pipe(csvParser.default())
            .on('data', (data) => items.push(data))
            .on('end', resolve)
    })
    await job
    return items
}


const removeResolverRecordFromInvalidRegistrarName = async (registrarRecords: RegistrarRecord[], resolverRecords: ResolverRecord[], resolverReverseRecords: ResolverReverseRecord[]) => {
    const registrarNames = registrarRecords.map((record) => record.name)
    const resolvers: NameKeyRecord[] = []
    resolverRecords.forEach((record) => {
        record.pairs.forEach((pair) => {
            resolvers.push({
                name: record.name,
                key: pair.key
            })
        })
    })
    const resolverReverses: NameKeyRecord[] = resolverReverseRecords.map((record) => {
        return {
            name: record.name,
            key: resolverReverseKey
        }
    })
    const nameNotIncludedResolverRecords = resolvers.filter((record) => !registrarNames.includes(record.name))
    logger.debug(`nameNotIncludedResolverRecords count : ${nameNotIncludedResolverRecords.length}`)

    const nameNotIncludedResolverReverseRecords = resolverReverses.filter((record) => !registrarNames.includes(record.name))
    logger.debug(`nameNotIncludedResolverReverseRecords count : ${nameNotIncludedResolverReverseRecords.length}`)
    const invalidResolverReverseRecords = resolverReverseRecords.filter((record) => registrarNames.includes(record.name))
        .filter((record) => {
            let registrar = registrarRecords.find((registrarRecord) => registrarRecord.name === record.name)
            return registrar?.owner !== record.value
        })
        .map((record) => {
            return {
                name: record.name,
                key: resolverReverseKey
            }
        })
    logger.debug(`invalidResolverReverseRecords count : ${invalidResolverReverseRecords.length}`)

    const notIncludedRecords = nameNotIncludedResolverRecords.concat(nameNotIncludedResolverReverseRecords)
    const operations: OperationRecord[] = notIncludedRecords.map((record) => {
        return {
            key: record.key,
            name: record.name,
            operation: "Remove",
            value: ''
        }
    })
    invalidResolverReverseRecords.forEach((record) => {
        operations.push({
            key: record.key,
            name: record.name,
            operation: "Remove",
            value: ''
        })
    })
    await saveOperationToCsv(operations, "RemoveResolverRecordFromInvalidRegistrarName")
}

const insertOrIgnoreDefaultValueForUninitializedRegistrarName = async (registrarRecords: RegistrarRecord[], resolverRecords: ResolverRecord[]) => {
    const registrarNames = registrarRecords.map((record) => record.name)
    const resolvers = resolverRecords.filter((record) => registrarNames.includes(record.name))
    logger.debug(`resolvers count : ${resolvers.length}`)
    const uninitializedAccountIdKeyResolvers = resolvers.filter((record) => {
        const keys = record.pairs.map((pair) => pair.key)
        return !keys.includes(accountIdKey)
    })
    const uninitializedPrincipalKeyResolvers = resolvers.filter((record) => {
        const keys = record.pairs.map((pair) => pair.key)
        return !keys.includes(principalKey)
    })
    const operations: OperationRecord[] = []
    uninitializedAccountIdKeyResolvers.forEach((record) => {
        let owner = registrarRecords.find((registrarRecord) => registrarRecord.name === record.name)?.owner
        if (owner) {
            operations.push({
                name: record.name,
                key: accountIdKey,
                operation: "InsertOrIgnore",
                value: owner
            })
        } else {
            logger.error(`can not find owner for name ${record.name}`)
            throw new Error(`can not find owner for name ${record.name}`)
        }
    })
    uninitializedPrincipalKeyResolvers.forEach((record) => {
        let owner = registrarRecords.find((registrarRecord) => registrarRecord.name === record.name)?.owner
        if (owner) {
            operations.push({
                name: record.name,
                key: principalKey,
                operation: "InsertOrIgnore",
                value: owner
            })
        } else {
            logger.error(`can not find owner for name ${record.name}`)
            throw new Error(`can not find owner for name ${record.name}`)
        }
    })
    await saveOperationToCsv(operations, "AddDefaultForRegistrar")
}

const upsertUserHasNoDefaultResolverReverse = async (registrarRecords: RegistrarRecord[], resolverReverseRecords: ResolverReverseRecord[]) => {
    const userGroup = groupBy(registrarRecords, (record) => record.owner)
    const operations: OperationRecord[] = []
    for (const [key, value] of Object.entries(userGroup)) {
        const user = key
        const names = value.map((record) => record.name)
        const resolverReverseRecordsForUser = resolverReverseRecords.filter((record) => names.includes(record.name))
        if (resolverReverseRecordsForUser.length === 0) {
            let name = names.sort()[0]
            operations.push({
                name: name,
                key: resolverReverseKey,
                operation: "Upsert",
                value: user
            })
        }

    }
    logger.debug(`upsertUserHasNoDefaultResolverReverse count : ${operations.length}`)
    await saveOperationToCsv(operations, `UpsertDefaultResolverReverseForUserAllReverseAreEmpty`)
}

const saveOperationToCsv = async (operations: OperationRecord[], fileName) => {
    // save records to csv
    const csvWriter = createObjectCsvWriter({
        path: './scripts/features/data/' + `${fileName}.csv`,
        header: [
            {id: 'name', title: 'name'},
            {id: 'operation', title: 'operation'},
            {id: 'key', title: 'key'},
            {id: 'value', title: 'value'},]
    })
    await csvWriter.writeRecords(operations)
        .then(() => {
            logger.log('...Done')
        })
}
const run = async () => {
    let registrarRecords = await readRegistrarCsv()
    logger.debug(`registrarRecords: ${JSON.stringify(registrarRecords.length)}`)
    let resolverRecords = await readResolverCsv()
    let resolverReverseRecords = await readResolverReverseCsv()
    await removeResolverRecordFromInvalidRegistrarName(registrarRecords, resolverRecords, resolverReverseRecords)
    await insertOrIgnoreDefaultValueForUninitializedRegistrarName(registrarRecords, resolverRecords)
    await upsertUserHasNoDefaultResolverReverse(registrarRecords, resolverReverseRecords)
}

(async () => {
    logger.debug('Start generate csv')
    await run()
})().then(() => {
    logger.info('csv generation done')
})
