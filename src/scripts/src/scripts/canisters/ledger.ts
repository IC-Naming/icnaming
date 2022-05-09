import '../setup'
import { canister, identity } from '../utils'
import { createActor } from '../declarations/ledger'
import fs from 'fs'
import { AccountIdentifier, SubAccount, TransferResult } from '~/declarations/ledger/ledger.did'
import { ReInstallOptions } from 'scripts/src/scripts/utils/canister'
import logger from 'node-color-log'

const { readFileSync, writeFileSync } = fs

const { identities } = identity

const name = 'ledger'

const dfx_file_path = 'dfx.json'

const reinstall_by_dfx = () => {
  const switch_to_public_did = () => {
    // load dfx.json file as json object
    const dfx_json = JSON.parse(readFileSync(dfx_file_path, 'utf8'))
    // get the public DID
    dfx_json.canisters[name].candid = 'scripts/wasm/ledger.public.did'
    // save the dfx.json file
    writeFileSync(dfx_file_path, JSON.stringify(dfx_json, null, 2))
  }

  const switch_to_init_did = () => {
    // load dfx.json file as json object
    const dfx_json = JSON.parse(readFileSync(dfx_file_path, 'utf8'))
    // get the public DID
    dfx_json.canisters[name].candid = 'scripts/wasm/ledger.did'
    // save the dfx.json file
    writeFileSync(dfx_file_path, `${JSON.stringify(dfx_json, null, 2)}\n`)
  }

  switch_to_init_did()

  const args = `'(record {
    send_whitelist = vec { };
    minting_account = "${identities.miner.account_id_hex}";
    transaction_window = opt record { secs = ${
        2 * 1024 * 1024
    } :nat64; nanos = 0:nat32};
    max_message_size_bytes = opt ${2 * 1024 * 1024} : opt nat64;
    archive_options = opt record {
        trigger_threshold = 12 : nat64;
        num_blocks_to_archive = 12 : nat64;
        node_max_memory_size_bytes =opt ${512 * 128} : opt nat64;
        max_message_size_bytes =opt ${2 * 1024 * 1024} : opt nat64;
        controller_id = principal "${identities.miner.principal_text}";
      };
    initial_values = vec { record { "${identities.main.account_id_hex}"; record { e8s = 100_000_000_000 : nat64; } } };
    })'`
  canister.reinstall('ledger', args)
  switch_to_public_did()
}

const add_some_data = async () => {
  const ledger_id = canister.get_id(name)
  const main_actor = createActor(ledger_id, {
    agentOptions: identities.main.agentOptions
  })
  const miner_actor = createActor(ledger_id, {
    agentOptions: identities.miner.agentOptions
  })

  const transfer_core = async (from_subaccount: [] | [SubAccount], to: AccountIdentifier, amount: bigint, memo: bigint, fee: bigint) => {
    const result: TransferResult = await main_actor.transfer({
      amount: {
        e8s: amount
      },
      fee: {
        e8s: fee
      },
      memo,
      from_subaccount,
      to,
      created_at_time: []
    })

    if ('Ok' in result) {
      logger.debug(`Transfer from ${from_subaccount} to ${to} with amount ${amount} and fee ${fee} succeeded`)
    } else if ('Err' in result) {
      logger.debug(result.Err)
    }
  }

  const transfer = async (to: AccountIdentifier, amount: bigint, memo: bigint, subaccount?: [SubAccount]) => {
    if (subaccount) {
      await transfer_core(subaccount, to, amount, memo, BigInt(10_000))
    } else {
      await transfer_core([], to, amount, memo, BigInt(10_000))
    }
  }

  const mint = async (to: AccountIdentifier, amount: bigint, memo: bigint) => {
    const result = await miner_actor.transfer({
      amount: {
        e8s: amount
      },
      fee: {
        e8s: BigInt(0)
      },
      memo,
      from_subaccount: [],
      to,
      created_at_time: []
    })
    if ('Ok' in result) {
      logger.debug(`Mint to ${to} with amount ${amount} and fee ${BigInt(0)} succeeded`)
    } else if ('Err' in result) {
      logger.debug(result.Err)
    }
  }

  const get_balance = async (account: number[]) => {
    const balance = await main_actor.account_balance({ account })
    logger.debug(`balance: ${balance.e8s}`)
  }

  const main = identities.main
  await get_balance(main.account_id_bytes)
  await get_balance(main.subaccount1_id_bytes)
  await get_balance(main.subaccount2_id_bytes)

  await mint(main.subaccount1_id_bytes, BigInt(100_000_000_000), BigInt(0))
  await transfer(main.subaccount1_id_bytes, BigInt(10_000_000_000), BigInt(1234))
  await transfer(main.subaccount2_id_bytes, BigInt(1_000_000_000), BigInt(1235), [identities.subaccount1])

  await get_balance(main.account_id_bytes)
  await get_balance(main.subaccount1_id_bytes)
  await get_balance(main.subaccount2_id_bytes)
}

export const reinstall = async (options?: ReInstallOptions) => {
  reinstall_by_dfx()
  // await add_some_data();
}
