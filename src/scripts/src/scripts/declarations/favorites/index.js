import { Actor, HttpAgent } from "@dfinity/agent";
import {actorFactory} from "../../utils/actorFactory";
import {favorites as name} from "../../canisters/names";
import { canister } from '@deland-labs/ic-dev-kit'

// Imports and re-exports candid interface
import { idlFactory } from './favorites.did.js';
export { idlFactory } from './favorites.did.js';
// CANISTER_ID is replaced by webpack based on node environment
export const canisterId = process.env.FAVORITES_CANISTER_ID;

/**
 *
 * @param {string | import("@dfinity/principal").Principal} canisterId Canister ID of Agent
 * @param {{agentOptions?: import("@dfinity/agent").HttpAgentOptions; actorOptions?: import("@dfinity/agent").ActorConfig}} [options]
 * @return {import("@dfinity/agent").ActorSubclass<import("./favorites.did.js")._SERVICE>}
 */
 export const createActor = (canisterId, options) => {
  const agent = new HttpAgent({ ...options?.agentOptions });

  // Fetch root key for certificate validation during development
  if(process.env.NODE_ENV !== "production") {
    agent.fetchRootKey().catch(err=>{
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
 * A ready-to-use agent for the favorites canister
 * @type {import("@dfinity/agent").ActorSubclass<import("./favorites.did.js")._SERVICE>}
 */
 export const favorites = actorFactory.createActor(idlFactory, canister.get_id(name));
 export const createFavorites = (identity_info) => createActor(canister.get_id(name), {agentOptions: identity_info.agentOptions});
