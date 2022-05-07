import { canister } from '~/utils'
import fs from 'fs'
import { identities } from '~/utils/identity'
import logger from 'node-color-log'
import { get_dfx_json } from '~/utils/dfx_json';

(async () => {
  await canister.create_all()
  const dfxJson = get_dfx_json()
  const names = dfxJson.canisters.keys()
  const dir = './env_configs'
  // create dir if not exists
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true })
  }

  let envFileContent = ''
  for (const name of names) {
    const envName = `NAMING_CANISTER_IDS_${name.toUpperCase()}`
    const value = canister.get_id(name)
    envFileContent += `export ${envName}=${value}\n`
  }
  // write env file
  fs.writeFileSync(`${dir}/dev.canister_ids.env`, envFileContent)

  // TODO: write env file for prod
  // let registrar_admin = `# main node\n${identities.main.principal_text}`;
  // fs.writeFileSync(`./configs/dev/principal_registrar_admin.in`, registrar_admin);
  // let state_exporter = `${registrar_admin}\n# state exporter node \n${identities.state_exporter.principal_text}`;
  // fs.writeFileSync(`./configs/dev/principal_state_exporter.in`, state_exporter);
  // let timer_trigger = `${registrar_admin}\n# timer_trigger node\n${identities.timer_trigger.principal_text}`;
  // fs.writeFileSync(`./configs/dev/principal_timer_trigger.in`, timer_trigger);
})()
