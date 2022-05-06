import "~/setup";
import logger from "node-color-log";
import { registrar } from "~/declarations/registrar";
import { resolver } from "~/declarations/resolver";
import { registry } from "~/declarations/registry";
import { favorites } from "~/declarations/favorites";
import fs from "fs";

// export state data
const export_state = async (actor: any, name: string) => {
    const now_time = new Date();
    const state_result = await actor.export_state();
    if ('Ok' in state_result) {
        const latest_actor_dir = `local_state_data/${name}`;
        if (!fs.existsSync(latest_actor_dir)) {
            fs.mkdirSync(latest_actor_dir, { recursive: true });
        }

        // update last file too
        const last_filename = `${latest_actor_dir}/latest.zlib`;
        fs.writeFileSync(last_filename, Uint8Array.from(state_result.Ok.state_data));
        logger.debug(`${name} State data exported, cost ${(new Date()).getTime() - now_time.getTime()}ms`);
    } else {
        logger.error(state_result.Err.message);
    }
}

(async () => {
    logger.debug('Start export state data');
    await Promise.all([
        export_state(registrar, "registrar"),
        export_state(resolver, "resolver"),
        export_state(registry, "registry"),
        export_state(favorites, "favorites")
    ]);
})().then(() => {
    logger.info("Starting canister build");
});
