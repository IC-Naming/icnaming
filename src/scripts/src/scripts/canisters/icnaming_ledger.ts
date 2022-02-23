import "../setup"
import {canister, convert, identity} from "../utils";
import {icnaming_ledger, icnaming_ledger as name, registrar} from "./names";
import {Principal} from "@dfinity/principal";
import {ReInstallOptions} from "~/utils/canister";
import {arrayOfNumberToUint8Array} from "~/utils/convert";

const identities = identity.identities;


const build = () => {
    canister.build(name)
}

const icnaming_canister_id = canister.get_id(icnaming_ledger);

export const get_quota_order_payment_receiver_subaccount_id = (): Array<number> => {
    let subaccount = arrayOfNumberToUint8Array(identities.registrar_quota_order_receiver_subaccount);
    return Array.from(convert.principalToAccountIDInBytes(Principal.fromText(icnaming_canister_id), subaccount));
}

export const get_quota_order_payment_refund_subaccount_id = (): Array<number> => {
    let subaccount = arrayOfNumberToUint8Array(identities.registrar_quota_order_refund_subaccount);
    return Array.from(convert.principalToAccountIDInBytes(Principal.fromText(icnaming_canister_id), subaccount));
}

const reinstall_by_dfx = async () => {
    await canister.reinstall_code(icnaming_ledger);
}

export const reinstall = async (options?: ReInstallOptions) => {
    if (options?.build) {
        build();
    }
    await reinstall_by_dfx();
}