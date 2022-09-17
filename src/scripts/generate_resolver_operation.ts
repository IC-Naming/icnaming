import logger from "node-color-log";
import {generateOperationCsv} from "~/generate_resolver_operation";

(async () => {
    logger.debug('Start generate csv')
    const startTime = performance.now()
    await generateOperationCsv();
    const endTime = performance.now()
    logger.debug(`generate resolver operation took ${endTime - startTime} milliseconds`)
})().then(() => {
    logger.info('csv generation done')
})
