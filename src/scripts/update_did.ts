import { exec } from 'shelljs'
import { favorites, registrar, registrar_control_gateway, registry, resolver } from '~/canisters/names'
import fs from 'fs'
import logger from 'node-color-log'

const downloadDid = async (canister) => {
  const result = exec(`dfx canister call ${canister} __get_candid_interface_tmp_hack`, { silent: true })
  if (result.code !== 0) {
    logger.error(result.stderr)
    process.exit(1)
  }
  const sourceContent = result.stdout
  // substring from first " to last "
  const start = sourceContent.indexOf('"') + 1
  const end = sourceContent.lastIndexOf('"')
  let didContent = sourceContent.substring(start, end)
  // replace \\n with \n
  didContent = didContent.replace(/\\n/g, '\n')
  return didContent
};

(async () => {
  const names = [registrar, registrar_control_gateway, registry, favorites, resolver]
  for (const name of names) {
    const didContent = await downloadDid(name)
    const didFile = `canisters/${name}/src/${name}.did`
    logger.debug(`Writing ${didFile}`)
    fs.writeFileSync(didFile, `${didContent}\n`)
  }

  logger.info('Did update complete')
})()
