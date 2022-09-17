import logger from "node-color-log";
import {
    RegistrarCsvLoader,
    ResolverCsvLoader,
    ResolverReverseNameCsvLoader
} from "~/generate_resolver_operation/loaders";
import {
    CreateDefaultResolverReverseNameRule,
    RegistrarWithoutFullResolver, ResolverKeys, ResolverReverseNameMatchingRule,
    ResolverWithoutRegistrarRule
} from "~/generate_resolver_operation/rules";
import {createObjectCsvWriter} from "csv-writer";


enum FileNames {
    RegistrarRecords = "RegistrarRecords.csv",
    ResolverReverseRecords = "ResolverReverseRecords.csv",
    ResolverRecords = "ResolverRecords.csv",
    ImportResolverRecordOperations = "ImportResolverRecordOperations.csv"
}

enum ResolverValueOperation {
    InsertOrIgnore = "InsertOrIgnore",
    Upsert = "Upsert",
    Remove = "Remove"
}

const getFilePath = (fileName: string): string => {
    return `./scripts/features/data/${fileName}`
}

const saveOperationToCsv = async (operations: OperationRecord[]) => {
    // save records to csv
    const csvWriter = createObjectCsvWriter({
        path: getFilePath(FileNames.ImportResolverRecordOperations),
        header: [
            {id: 'name', title: 'name'},
            {id: 'operation', title: 'operation'},
            {id: 'key', title: 'key'},
            {id: 'value', title: 'value'},]
    })
    await csvWriter.writeRecords(operations)
}

interface OperationRecord {
    name: string,
    key: string,
    value: string,
    operation: ResolverValueOperation
}

export const generateOperationCsv = async () => {
    let registrarCsvLoader = new RegistrarCsvLoader(getFilePath(FileNames.RegistrarRecords));
    await registrarCsvLoader.Load();

    let resolverReverseCsvLoader = new ResolverReverseNameCsvLoader(getFilePath(FileNames.ResolverReverseRecords));
    await resolverReverseCsvLoader.Load();

    let resolverCsvLoader = new ResolverCsvLoader(getFilePath(FileNames.ResolverRecords));
    await resolverCsvLoader.Load();


    let operations: OperationRecord[] = [];

    {
        let rule = new ResolverWithoutRegistrarRule();
        let resolversNeedTobeDelete = rule.Check(registrarCsvLoader.registrars, resolverCsvLoader.resolvers);
        resolversNeedTobeDelete.forEach(resolver => {
            resolver.values.forEach((value, key) => {
                operations.push({
                    name: resolver.name,
                    key,
                    value,
                    operation: ResolverValueOperation.Remove
                })
            });
        });
    }

    {
        let rule = new RegistrarWithoutFullResolver();
        let resolversNeedToAppend = rule.Check(registrarCsvLoader.registrars, resolverCsvLoader.resolvers);
        resolversNeedToAppend.forEach(item => {
            operations.push({
                name: item.registrar.name,
                key: item.key,
                value: item.value,
                operation: ResolverValueOperation.InsertOrIgnore
            })
        });
    }

    {
        let rule = new CreateDefaultResolverReverseNameRule();
        let reverseNameNeedTobeCreated = rule.Check(registrarCsvLoader.userNames, resolverReverseCsvLoader.resolverReverseNames);
        reverseNameNeedTobeCreated.forEach(item => {
            operations.push({
                name: item.defaultReverseName.name,
                key: ResolverKeys.resolverReverseKey,
                value: item.defaultReverseName.owner,
                operation: ResolverValueOperation.InsertOrIgnore
            })
        });
    }
    {

        let rule = new ResolverReverseNameMatchingRule();
        let reverseNameNeedTobeCreated = rule.Check(registrarCsvLoader.registrars, resolverReverseCsvLoader.resolverReverseNames);
        reverseNameNeedTobeCreated.forEach(item => {
            operations.push({
                name: item.resolverReverseName.name,
                key: ResolverKeys.resolverReverseKey,
                value: "",
                operation: ResolverValueOperation.Remove
            })
        });
    }

    await saveOperationToCsv(operations);

}