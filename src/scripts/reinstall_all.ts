import { reinstall_all } from './src/tasks'
import logger from 'node-color-log';

(async () => {
  await reinstall_all({
    build: true,
    init: true,
    canisters: {
      ledger: true,
      registrar: true,
      registrar_control_gateway: true,
      registry: true,
      resolver: true,
      favorites: true,
      cycles_minting: true,
      dicp: true,
    }
  })
})().then(() => {
  logger.info('reinstall_all.ts: All done.')
}).catch((err) => {
  console.error('reinstall_all.ts: Error:', err)
})
