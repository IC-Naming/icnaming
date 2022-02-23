import "../setup"
import {canister} from "../utils";
import {favorites as name} from "./names";
import {ReInstallOptions} from "scripts/src/scripts/utils/canister";


const build = () => {
    canister.build(name)
}

const reinstall_by_dfx = async () => {
    await canister.reinstall_code(name);
}

export const reinstall = async (options?: ReInstallOptions) => {
    if (options?.build) {
        build();

    }
    await reinstall_by_dfx();
}