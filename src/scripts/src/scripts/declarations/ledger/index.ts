import {Actor, HttpAgent} from "@dfinity/agent";
import {actorFactory} from "../../utils/actorFactory";
import {ledger as name} from "../../canisters/names";
import {canister} from "@deland-labs/ic_ledger_client";

// Imports and re-exports candid interface
import {idlFactory} from '@deland-labs/ic_ledger_client';
// CANISTER_ID is replaced by webpack based on node environment
export const canisterId = process.env.LEDGER_CANISTER_ID;

export const createActor = (canisterId, options) => {
    const agent = new HttpAgent({...options?.agentOptions});

    // Fetch root key for certificate validation during development
    if (process.env.NODE_ENV !== "production") {
        agent.fetchRootKey().catch(err => {
            console.warn("Unable to fetch root key. Check to ensure that your local replica is running");
            console.error(err);
        });
    }

    // Creates an actor with using the candid interface and the HttpAgent
    return Actor.createActor(idlFactory, {
        agent,
        canisterId,
        ...options?.actorOptions,
    });
};

/**
 * A ready-to-use agent for the ledger canister
 * @type {import("@dfinity/agent").ActorSubclass<import("./ledger.did.js")._SERVICE>}
 */
export const ledger = actorFactory.createActor(idlFactory, canister.get_id(name));
export const createLedger = (identity_info) => createActor(canister.get_id(name), {agentOptions: identity_info.agentOptions});
