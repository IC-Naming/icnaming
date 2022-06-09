import { Actor, HttpAgent } from "@dfinity/agent";
import { actorFactory } from "../../utils/actorFactory";
import { dicp as name } from "../../canisters/names";
import { canister, identity } from '@deland-labs/ic-dev-kit'

// Imports and re-exports candid interface
import { idlFactory } from '@deland-labs/dft_all_features_client'
// CANISTER_ID is replaced by webpack based on node environment
export const canisterId = process.env.DICP_CANISTER_ID;

export const createActor = (canisterId, options) => {
  const agent = new HttpAgent({ ...options?.agentOptions });

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
 * A ready-to-use agent for the dicp canister
 * @type {import("@dfinity/agent").ActorSubclass<import("./dicp.did.js")._SERVICE>}
 */
export const dicp = actorFactory.createActor(idlFactory, canister.get_id(name), identity.identityFactory.getIdentity()!);
export const createDicp = (identity_info) => createActor(canister.get_id(name), { agentOptions: identity_info.agentOptions });
