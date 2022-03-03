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

    let registrar_admin = `# main node\n${identities.main.principal_text}`;
    fs.writeFileSync(`./configs/dev/principal_registrar_admin.in`, registrar_admin);
    let state_exporter = `${registrar_admin}\n# state exporter node \n${identities.state_exporter.principal_text}`;
    fs.writeFileSync(`./configs/dev/principal_state_exporter.in`, state_exporter);
    let timer_trigger = `${registrar_admin}\n# timer_trigger node\n${identities.timer_trigger.principal_text}`;
    fs.writeFileSync(`./configs/dev/principal_timer_trigger.in`, timer_trigger);
})();