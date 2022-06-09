import fs from "fs";
import archiver from "archiver";
import {DfxJsonCanister, DfxPackageEnv, get_dfx_json, get_dfx_package_json, get_wasm_path} from "~/utils/dfx_json";
import {canister} from "~/utils";
import logger from "node-color-log";

const package_dir = "package"
// dir to save packages build by diff feature
const package_canister_env_dir = "package_canister_env"

const build_all = async (build_context: BuildContext) => {
    // reset package_feature dir
    if (fs.existsSync(package_canister_env_dir)) {
        fs.rmSync(package_canister_env_dir, {recursive: true})
    }
    fs.mkdirSync(package_canister_env_dir)

    // distinct feature
    let canister_envs = build_context.canister_envs;

    const declarations_target_dir = `${package_canister_env_dir}/declarations`
    fs.mkdirSync(declarations_target_dir, {recursive: true})

    for (let [name, canister_json] of Object.entries(build_context.canisters)) {

        // copy copy_ts_declarations
        if (canister_json.pack_config?.copy_ts_declarations != false) {
            const source_dir = `./scripts/src/scripts/declarations/${name}`
            fs.cpSync(source_dir, `${declarations_target_dir}/${name}`, {recursive: true, force: true})
        }
    }

    // build each canister by each feature
    for (const canisterEnv of canister_envs) {
        // make a feature dif
        const canister_env_dir = `${package_canister_env_dir}/${canisterEnv}`
        fs.mkdirSync(canister_env_dir)

        logger.debug(`build canister_env: ${canisterEnv}`);
        for (let [name, canister_json] of Object.entries(build_context.canisters)) {
            canister.build(name, canisterEnv);
            // copy wasm files to feature dir
            const wasm_path = get_wasm_path(canister_json);
            fs.copyFileSync(wasm_path, `${canister_env_dir}/${name}.wasm`);

            // copy did files to feature dir
            const did_path = canister_json.candid;
            fs.copyFileSync(did_path, `${canister_env_dir}/${name}.did`);
        }
    }
}

const clean = async () => {
    const found = fs.existsSync(package_dir);
    if (found) {
        logger.info("Cleaning package directory")
        fs.rmSync(package_dir, {recursive: true});
    }

    fs.mkdirSync(package_dir);
}

const check = async (build_context: BuildContext) => {
    // ensure every wasm file in package_feature dir must be < 2MB, check recursive
    for (let feature of build_context.canister_envs) {
        const feature_dir = `${package_canister_env_dir}/${feature}`
        const files = fs.readdirSync(feature_dir);
        for (const file of files) {
            if (file.endsWith(".wasm")) {
                const file_path = `${feature_dir}/${file}`
                const stat = fs.statSync(file_path);
                if (stat.size > 2 * 1024 * 1024) {
                    logger.warn(`WASM file size of ${file} is ${stat.size} bytes, must be < 2MB`);
                }
            }
        }
    }

    logger.debug("Check passed")
}

const create = async (build_context: BuildContext) => {

    const out_dfx_json = {
        "defaults": {
            "build": {
                "args": "",
                "packtool": ""
            }
        },
        "networks": {
            "local": {
                "bind": "127.0.0.1:8000",
                "type": "ephemeral"
            },
            "ic": {
                "providers": ["https://ic0.app"],
                "type": "persistent"
            },
            "staging": {
                "providers": ["https://ic0.app"],
                "type": "persistent"
            },
        },
        "version": 1
    };

    const canister_node = {};

    for (const name of Object.keys(build_context.canisters)) {
        canister_node[name] = {
            "candid": `assets/${name}.did`,
            "wasm": `assets/${name}.wasm`,
            "type": "custom"
        };
    }
    out_dfx_json["canisters"] = canister_node;

    logger.debug("creating package for each env");
    for (const env of build_context.envs) {
        logger.debug(`creating package for env: ${env.name}`);
        const env_dir = `${package_dir}/${env.name}`;
        fs.mkdirSync(env_dir);

        // copy canister_ids
        const source_canister_ids_json = `./canister_ids.json`;
        const dest_canister_ids_json = `${env_dir}/canister_ids.json`;
        fs.copyFileSync(source_canister_ids_json, dest_canister_ids_json);
        logger.debug(`copy canister_ids.json from ${source_canister_ids_json} to ${dest_canister_ids_json}`);

        // copy assets
        const env_assets_dir = `${env_dir}/assets`;
        fs.mkdirSync(env_assets_dir);

        // copy files from package_feature dir to env dir
        let canister_env = env.canister_env;
        const canister_env_dir = `${package_canister_env_dir}/${canister_env}`;
        fs.cpSync(canister_env_dir, env_assets_dir, {recursive: true, force: true})

        // copy declarations
        const env_declarations_dir = `${env_dir}/scripts/src/scripts/declarations`;
        fs.mkdirSync(env_declarations_dir, {recursive: true});
        fs.cpSync(`${package_canister_env_dir}/declarations`, env_declarations_dir, {recursive: true, force: true})

        // out dfx.json
        const dest_dfx_json = `${env_dir}/dfx.json`;
        fs.writeFileSync(dest_dfx_json, JSON.stringify(out_dfx_json, null, 2));
        logger.debug(`Created dfx.json for ${env.name}`);

        // create ${env}.env file
        const dest_env_file = `${env_dir}/${env.name}.env`;
        fs.writeFileSync(dest_env_file, env.name);
        logger.debug(`Created ${env.name}.env for ${env.name}`);

        logger.info(`Created package for ${env.name}`);
    }
}

const create_zip = async (build_context: BuildContext) => {
    // create zip file for each env
    for (const env of build_context.envs) {
        const env_dir = `${package_dir}/${env.name}`;
        const output_zip = fs.createWriteStream(`${env_dir}.zip`);
        const archive = archiver("zip", {
            zlib: {level: 9}
        });

        archive.pipe(output_zip);
        archive.directory(env_dir, false);
        await archive.finalize();

        logger.info(`Created zip file for ${env.name}`);
    }
}

interface BuildContext {
    canisters: Map<string, DfxJsonCanister>
    envs: DfxPackageEnv[],
    canister_envs: string[]
}

(async () => {
    const dfxJson = get_dfx_json();
    const dfxPackageJson = get_dfx_package_json();
    // join canisters keys as string
    const canisters_keys = Array.from(dfxJson.canisters.keys()).join(", ");
    logger.info(`There are canister listed in dfx.json: ${canisters_keys}`);

    // filter canister those not exclude in package
    const exclude_canisters: string[] = [];
    const canisters = {};

    for (const [name, canister] of dfxJson.canisters.entries()) {
        if (canister.pack_config?.exclude_in_package) {
            exclude_canisters.push(name);
            continue;
        }
        canisters[name] = canister;
    }

    if (exclude_canisters.length > 0) {
        logger.info(`Exclude canisters: ${exclude_canisters.join(", ")}`);
    }

    const build_context: BuildContext = {
        canisters: canisters as Map<string, DfxJsonCanister>,
        envs: dfxPackageJson.envs,
        canister_envs: [...new Set(dfxPackageJson.envs.map(env => env.canister_env))]
    };
    logger.debug(`build_context: ${JSON.stringify(build_context, null, 2)}`);

    await clean();
    await build_all(build_context);
    await check(build_context);
    await create(build_context);
    await create_zip(build_context);

})().then(() => {
    logger.debug("Package created successfully");
})
