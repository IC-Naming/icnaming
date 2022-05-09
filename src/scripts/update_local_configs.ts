import { canister } from '~/utils'
import fs from 'fs'
import { identities } from '~/utils/identity'
import {get_dfx_json} from "~/utils/dfx_json";

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

  const principalContent = `export NAMING_PRINCIPAL_NAME_ADMIN="
# main node
${identities.main.principal_text}
"
export NAMING_PRINCIPAL_NAME_STATE_EXPORTER="
# main node
${identities.main.principal_text}
# state exporter node
${identities.state_exporter.principal_text}
"
export NAMING_PRINCIPAL_NAME_TIMER_TRIGGER="
# main node
${identities.main.principal_text}
# timer_trigger node
${identities.timer_trigger.principal_text}
"
`
  fs.writeFileSync(`${dir}/dev.principals.env`, principalContent)
})()
