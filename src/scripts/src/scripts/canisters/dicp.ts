import { canister, identity, canisterInit } from '@deland-labs/ic-dev-kit';
import { ReInstallOptions } from '~/utils/canister'

const build = () => {
  canister.build('dicp')
}

export const reinstall = async (options?: ReInstallOptions) => {
  if (options?.build) {
    build()
  }

  const args = canisterInit.parseDFTInit({
    name: "dicp",
    symbol: "DICP",
    total_supply: "100000000000000000",
    fee: {
      minimum: "0",
      rate: 0,
      rateDecimals: 0
    },
    decimals: 8,
    caller: identity.identityFactory.getPrincipal()?.toText()
  });

  await canister.reinstall_code("dicp", args);

}
