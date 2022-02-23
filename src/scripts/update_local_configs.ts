import {canister} from "~/utils";
import fs from "fs";
import {identities} from "~/utils/identity";
import logger from "node-color-log";

(async () => {
    await canister.create_all();
    const names = ["registrar", "resolver", "registry", "icnaming_ledger", "cycles_minting", "favorites", "ledger"]
    for (let name of names) {
        let id = canister.get_id(name);
        fs.writeFileSync(`./configs/dev/canister_ids_${name}.in`, id);
    }

    logger.debug("local canister ids updated");

    // out identities.main.principal_text to principal_registrar_admin.in
    fs.writeFileSync(`./configs/dev/principal_registrar_admin.in`, identities.main.principal_text);
})();