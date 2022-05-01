import "../setup"
import {canister, convert, identity} from "../utils";
import {ReInstallOptions} from "~/utils/canister";
import logger from "node-color-log";

const identities = identity.identities;

const build = () => {
    canister.build("dicp");
}

const reinstall_by_dfx = async (args: string) => {
    await canister.reinstall("dicp", args);
}

export const reinstall = async (options?: ReInstallOptions) => {
    if (options?.build) {
        build();
    }
    const name = "dicp";
    const symbol = "DICP";
    const decimals = 8;
    const supply = "1_000_000_000_00000000"

    const archiveArgs = "null";

    let owner_principal = identities.main.identity.getPrincipal().toText();
    const owner = `opt principal "${owner_principal}"`;
    const args = `'(null ,null ,"${name}", "${symbol}", ${decimals}:nat8, ${supply}:nat, record { minimum = 0 : nat; rate = 0 : nat32; rateDecimals= 0:nat8 } , ${owner}, ${archiveArgs})'`;
    logger.debug(`Reinstall by dfx: ${args}`);
    await reinstall_by_dfx(args);
}
