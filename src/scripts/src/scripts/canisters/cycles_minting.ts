import "../setup"
import {canister} from "../utils";
import {cycles_minting as name} from "./names";
import {ReInstallOptions} from "~/utils/canister";
import {identities} from "~/utils/identity";


const build = () => {
    canister.build(name)
}

const reinstall_by_dfx = async () => {
    await canister.reinstall_code(name);

}
const init = () => {
}

export const reinstall = async (options?: ReInstallOptions) => {
    if (options?.build) {
        build();

    }
    await reinstall_by_dfx();

    if (options?.init) {
        init();
    }
}