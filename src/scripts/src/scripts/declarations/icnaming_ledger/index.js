import { Actor, HttpAgent } from "@dfinity/agent";
import {actorFactory} from "../../utils/actorFactory";
import {icnaming_ledger as name} from "../../canisters/names";
import {canister} from "../../utils";


// Imports and re-exports candid interface
import { idlFactory } from './icnaming_ledger.did.js';
export { idlFactory } from './icnaming_ledger.did.js';
// CANISTER_ID is replaced by webpack based on node environment
export const canisterId = process.env.ICNAMING_LEDGER_CANISTER_ID;

/**
 * 
 * @param {string | import("@dfinity/principal").Principal} canisterId Canister ID of Agent
 * @param {{agentOptions?: import("@dfinity/agent").HttpAgentOptions; actorOptions?: import("@dfinity/agent").ActorConfig}} [options]
 * @return {import("@dfinity/agent").ActorSubclass<import("./icnaming_ledger.did.js")._SERVICE>}
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
 * A ready-to-use agent for the icnaming_ledger canister
 * @type {import("@dfinity/agent").ActorSubclass<import("./icnaming_ledger.did.js")._SERVICE>}
 */
 export const icnaming_ledger = actorFactory.createActor(idlFactory, canister.get_id(name));
