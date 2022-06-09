npm config set @deland-labs:registry https://www.myget.org/F/ic-feed/npm/
npm install
npx icdev init-identity
ts-node -r tsconfig-paths/register scripts/update_local_configs.ts
