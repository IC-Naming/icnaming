import "~/setup"
import {reinstall_all} from "./src/tasks"
import logger from "node-color-log";

(async () => {
    await reinstall_all({
        build: true,
        init: true,
        canisters: {
            ledger: true,
            registrar: true,
            icnaming_ledger: true,
            registry: true,
            resolver: true,
            favorites: true,
        }
    });
})().then(() => {
    logger.info("reinstall_all.ts: All done.");
}).catch((err) => {
    console.error("reinstall_all.ts: Error:", err);
});