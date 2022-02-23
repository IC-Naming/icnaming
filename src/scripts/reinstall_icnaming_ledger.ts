import {reinstall} from "~/canisters/icnaming_ledger";
import logger from "node-color-log";

reinstall().then(() => {
    logger.info("ICNaming Ledger reinstalled");
}).catch((err) => {
    console.error("ICNaming Ledger reinstall failed: " + err);
});