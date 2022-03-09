import "~/setup"

import fs from "fs";
import logger from "node-color-log";
import {registrar} from "~/declarations/registrar";
import {registry} from "~/declarations/registry";
import {resolver} from "~/declarations/resolver";
import {favorites} from "~/declarations/favorites";


(async () => {
    await registrar.run_tasks();
})().then(() => {
    logger.info("Finished running tasks");
});