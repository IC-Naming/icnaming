import "~/setup"

import fs from "fs";
import logger from "node-color-log";
import {registrar} from "~/declarations/registrar";
import {registry} from "~/declarations/registry";
import {resolver} from "~/declarations/resolver";
import {favorites} from "~/declarations/favorites";

let state_dir = "latest_state_data";

const load_state = async (actor: any, file_path: string) => {
    let latest_state_file = `${state_dir}/${file_path}`;
    let latest_state_data = fs.readFileSync(latest_state_file);
    let load_state_result = await actor.load_state({
        state_data: Array.from(latest_state_data)
    });
    if ('Ok' in load_state_result) {
        logger.info(`Loaded state from ${latest_state_file}`);
    } else {
        logger.error(`Failed to load state from ${latest_state_file}, error: ${JSON.stringify(load_state_result)}`);
    }
}

(async () => {
    await Promise.all([
        load_state(registrar, "registrar/latest.zlib"),
        load_state(registry, "registry/latest.zlib"),
        load_state(resolver, "resolver/latest.zlib"),
        load_state(favorites, "favorites/latest.zlib"),
    ]);
})().then(() => {
    logger.info("state loaded");
});