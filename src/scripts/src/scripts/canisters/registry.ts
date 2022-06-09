import { canister } from '@deland-labs/ic-dev-kit'
import { registry as name } from './names'
import { ReInstallOptions } from '~/utils/canister'
import { reinstall_with_dev_ids } from './installUtils'

const build = () => {
  canister.build(name)
}

export const reinstall = async (options?: ReInstallOptions) => {
  if (options?.build) {
    build()
  }
  await reinstall_with_dev_ids(name)
}
