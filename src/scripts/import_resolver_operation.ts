import logger from "node-color-log";
import {import_record_value_from_csv} from "./features/step_definitions/utils";


(async () => {
    logger.debug('Start import resolver record operations csv')
    await import_record_value_from_csv('ImportResolverRecordOperations')
})().then(() => {
    logger.info('import resolver record operations done')
})
