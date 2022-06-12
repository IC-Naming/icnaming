import fs from 'fs'
import { identity, dfxJson, canister } from '@deland-labs/ic-dev-kit';

(async () => {
  const names = dfxJson.get_dfx_json().canisters.keys()
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
${identity.identityFactory.getPrincipal()?.toText()}
"
export NAMING_PRINCIPAL_NAME_STATE_EXPORTER="
# main node
${identity.identityFactory.getPrincipal()?.toText()}
# state exporter node
${identity.identityFactory.getPrincipal("icnaming_state_exporter")?.toText()}
"
export NAMING_PRINCIPAL_NAME_TIMER_TRIGGER="
# main node
${identity.identityFactory.getPrincipal()?.toText()}
# timer_trigger node
${identity.identityFactory.getPrincipal("icnaming_timer_trigger")?.toText()}
"
`
  fs.writeFileSync(`${dir}/dev.principals.env`, principalContent)
})()
