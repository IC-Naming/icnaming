import "./scripts/setup"
import {reinstall as reinstall_ledger} from "./scripts/canisters/ledger";
import {reinstall as reinstall_icnaming_ledger} from "./scripts/canisters/icnaming_ledger";
import {reinstall as reinstall_favorites} from "./scripts/canisters/favorites";
import {reinstall as reinstall_registrar} from "./scripts/canisters/registrar";
import {reinstall as reinstall_registrar_control_gateway} from "./scripts/canisters/registrar_control_gateway";
import {reinstall as reinstall_resolver} from "./scripts/canisters/resolver";
import {reinstall as reinstall_registry} from "./scripts/canisters/registry";
import {reinstall as reinstall_cycles_minting} from "./scripts/canisters/cycles_minting";
import {reinstall as reinstall_dicp} from "./scripts/canisters/dicp";


export const reinstall_all = async (options?: CanisterReinstallOptions) => {
    // recode time of cost
    const start = Date.now();

    if (options && options.one_by_one) {
        if (options && options.canisters?.ledger) {
            await reinstall_ledger({
                ...options,
            });
        }

        if (options && options.canisters?.dicp) {
            await reinstall_dicp({
                ...options,
            });
        }

        if (options && options.canisters?.icnaming_ledger) {
            await reinstall_icnaming_ledger({
                ...options,
            });
        }

        if (options && options.canisters?.favorites) {
            await reinstall_favorites({
                ...options,
            });
        }

        if (options && options.canisters?.registrar) {
            await reinstall_registrar({
                ...options,
            });
        }

        if (options && options.canisters?.registrar_control_gateway) {
            await reinstall_registrar_control_gateway({
                ...options,
            });
        }

        if (options && options.canisters?.resolver) {
            await reinstall_resolver({
                ...options,
            });
        }

        if (options && options.canisters?.registry) {
            await reinstall_registry({
                ...options,
            });
        }

        if (options && options.canisters?.cycles_minting) {
            await reinstall_cycles_minting({
                ...options,
            });
        }

    } else {
        console.info("reinstall all in parallel");
        let jobs: Promise<void>[] = [];
        if (options && options.canisters?.ledger) {
            jobs.push(reinstall_ledger({
                ...options,
            }));
        }

        if (options && options.canisters?.dicp) {
            jobs.push(reinstall_dicp({
                ...options,
            }));
        }

        if (options && options.canisters?.icnaming_ledger) {
            jobs.push(reinstall_icnaming_ledger({
                ...options,
            }));
        }

        if (options && options.canisters?.favorites) {
            jobs.push(reinstall_favorites({
                ...options,
            }));
        }

        if (options && options.canisters?.registrar) {
            jobs.push(reinstall_registrar({
                ...options,
            }));
        }

        if (options && options.canisters?.registrar_control_gateway) {
            jobs.push(reinstall_registrar_control_gateway({
                ...options,
            }));
        }

        if (options && options.canisters?.resolver) {
            jobs.push(reinstall_resolver({
                ...options,
            }));
        }

        if (options && options.canisters?.registry) {
            jobs.push(reinstall_registry({
                ...options,
            }));
        }

        if (options && options.canisters?.cycles_minting) {
            jobs.push(reinstall_cycles_minting({
                ...options,
            }));
        }

        await Promise.all(jobs);
    }

    const end = Date.now();
    console.info(`reinstall all in ${end - start} ms`);
    // sleep for 3 seconds to waiting code to be available
    await new Promise((resolve) => setTimeout(resolve, 3000));
}

export interface CanisterReinstallOptionsCanisters {
    ledger?: boolean;
    dicp?: boolean;
    icnaming_ledger?: boolean;
    favorites?: boolean;
    registrar?: boolean;
    registrar_control_gateway?: boolean;
    resolver?: boolean;
    registry?: boolean;
    cycles_minting?: boolean;
}

export interface CanisterReinstallOptions {
    build?: boolean;
    init?: boolean;
    one_by_one?: boolean;
    canisters?: CanisterReinstallOptionsCanisters;
}
