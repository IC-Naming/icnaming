import {canister} from "~/utils";
import fs from "fs";
import {identities} from "~/utils/identity";
import logger from "node-color-log";

(async () => {
    await canister.create_all();
    const names = ["registrar", "resolver", "registry", "icnaming_ledger", "cycles_minting", "favorites", "ledger"]
    let dir = `./configs/dev`;
    // create dir if not exists
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, {recursive: true});
    }

    for (let name of names) {
        let id = canister.get_id(name);
        let file = `${dir}/canister_ids_${name}.in`;
        fs.writeFileSync(file, id);
    }

    logger.debug("local canister ids updated");

    // out identities.main.principal_text to principal_registrar_admin.in
    fs.writeFileSync(`./configs/dev/principal_registrar_admin.in`, identities.main.principal_text);
})();