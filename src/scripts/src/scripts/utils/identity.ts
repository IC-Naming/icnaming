import { exec } from 'shelljs'
import { Identity } from '@dfinity/agent'
import fs from 'fs'
import { Secp256k1KeyIdentity } from '@dfinity/identity'
import sha256 from 'sha256'
import { principalToAccountIDInBytes, toHexString } from './convert'
import { Principal } from '@dfinity/principal'
import { all_names } from '~/canisters/names'
import { get_id } from '~/utils/canister'

function get_pem_path (name: string): string {
  const home = process.env.HOME
  return `${home}/.config/dfx/identity/${name}/identity.pem`
}

export function load (name: string): Identity {
  new_dfx_identity(name)
  const pem_path = get_pem_path(name)
  const rawKey = fs
    .readFileSync(pem_path)
    .toString()
    .replace('-----BEGIN EC PRIVATE KEY-----', '')
    .replace('-----END EC PRIVATE KEY-----', '')
    .trim()

  // @ts-ignore
  const rawBuffer = Uint8Array.from(rawKey).buffer

  const privKey = Uint8Array.from(sha256(rawBuffer, { asBytes: true }))

  // Initialize an identity from the secret key
  return Secp256k1KeyIdentity.fromSecretKey(
    Uint8Array.from(privKey).buffer
  )
}

export const new_dfx_identity = (name: string) => {
  exec(`dfx identity new ${name}`, { silent: true })
  // override static key file from scripts/identity_pem/${name}/identity.pem
  const target_pem_path = get_pem_path(name)
  const source_pem_path = `scripts/identity_pem/${name}/identity.pem`
  // chmod 777 target_pem_path
  fs.chmodSync(target_pem_path, '777')
  fs.copyFileSync(source_pem_path, target_pem_path)
}

export const use_dfx_identity = (name: string) => {
  exec(`dfx identity use ${name}`, { silent: true })
}

export interface IdentityDfxInfo {
    principal_text: string;
    account_id: string;
}

export interface agentOptions {
    host: string;
    identity: Identity;
}

export interface IdentityInfo {
    identity: Identity;
    principal_text: string;
    agentOptions: agentOptions;
    account_id_hex: string;
    account_id_bytes: number[];
    subaccount1_id_bytes: number[];
    subaccount2_id_bytes: number[];
    subaccount3_id_bytes: number[];
}

export interface IdentityCollection {
    main: IdentityInfo;
    miner: IdentityInfo,
    user1: IdentityInfo,
    user2: IdentityInfo,
    user3: IdentityInfo,
    state_exporter: IdentityInfo,
    timer_trigger: IdentityInfo,
    subaccount1: number[],
    subaccount2: number[],
    subaccount3: number[],
    // subaccount for icnaming ledger to receive quota order payment
    registrar_quota_order_receiver_subaccount: number[],
    // subaccount for icnaming ledger to refund quota order payment
    registrar_quota_order_refund_subaccount: number[],

    get_identity_info(name: string): IdentityInfo;

    get_principal(name: string): Principal
}

export const create_identities = () => {
  new_dfx_identity('icnaming_main')
  new_dfx_identity('icnaming_miner')
  new_dfx_identity('icnaming_user1')
  new_dfx_identity('icnaming_user2')
  new_dfx_identity('icnaming_user3')
  new_dfx_identity('icnaming_state_exporter')
  new_dfx_identity('icnaming_timer_trigger')
}

export const identities = ((): IdentityCollection => {
  const get_subaccount = (index: number) => {
    const subAccount = new Uint8Array(32).fill(0)
    subAccount[0] = index
    return subAccount
  }

  const create_identities = (name: string): IdentityInfo => {
    const identity = load(name)
    const principal = identity.getPrincipal()
    const account_id_uint8 = principalToAccountIDInBytes(principal)
    const account_id_bytes = Array.from(account_id_uint8)
    return {
      identity,
      principal_text: principal.toText(),
      agentOptions: {
        host: 'http://127.0.0.1:8000',
        identity
      },
      account_id_hex: toHexString(account_id_uint8),
      account_id_bytes,
      subaccount1_id_bytes: Array.from(principalToAccountIDInBytes(principal, (get_subaccount(1)))),
      subaccount2_id_bytes: Array.from(principalToAccountIDInBytes(principal, (get_subaccount(2)))),
      subaccount3_id_bytes: Array.from(principalToAccountIDInBytes(principal, (get_subaccount(3))))
    }
  }

  const default_identities = create_identities('icnaming_main')
  const miner_identities = create_identities('icnaming_miner')
  const user1_identities = create_identities('icnaming_user1')
  const user2_identities = create_identities('icnaming_user2')
  const user3_identities = create_identities('icnaming_user3')
  const state_exporter_identities = create_identities('icnaming_state_exporter')
  const timer_trigger_identities = create_identities('icnaming_timer_trigger')

  // reset to default in the end
  use_dfx_identity('icnaming_main')

  return {
    main: default_identities,
    miner: miner_identities,
    user1: user1_identities,
    user2: user2_identities,
    user3: user3_identities,
    state_exporter: state_exporter_identities,
    timer_trigger: timer_trigger_identities,
    subaccount1: Array.from(get_subaccount(1)),
    subaccount2: Array.from(get_subaccount(2)),
    subaccount3: Array.from(get_subaccount(3)),
    registrar_quota_order_receiver_subaccount: Array.from(get_subaccount(0x11)),
    registrar_quota_order_refund_subaccount: Array.from(get_subaccount(0x12)),
    get_identity_info (name: string): IdentityInfo {
      return this[name]
    },
    get_principal (name: string): Principal {
      return get_principal(name)
    }
  }
})()

export const identities_to_json = (identities: IdentityCollection): string => {
  // serialize identities as json
  // if property is Array<number>, convert to hex string
  // ignore agentOptions
  return JSON.stringify(identities, (key, value) => {
    if (key === 'agentOptions') {
      return undefined
    }
    if (key === 'identity') {
      return undefined
    }
    if (Array.isArray(value)) {
      return toHexString(Uint8Array.from(value))
    }
    return value
  }, 2)
}

/**
 * get principal by name.
 * @param name canister name, identity name, principal text, "anonymous"
 */
export const get_principal = (name: string): Principal => {
  if (name == 'anonymous') {
    return Principal.anonymous()
  }
  const identityInfo = identities.get_identity_info(name)
  let user_principal: Principal
  if (identityInfo != null) {
    user_principal = identityInfo.identity.getPrincipal()
  } else {
    // all_names contains name
    if (all_names.includes(name)) {
      user_principal = Principal.fromText(get_id(name))
    } else {
      user_principal = Principal.fromText(name)
    }
  }
  return user_principal
}
