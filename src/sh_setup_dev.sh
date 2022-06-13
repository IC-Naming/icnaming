npm config set @deland-labs:registry https://www.myget.org/F/ic-feed/npm/
npm config set @icnaming:registry https://www.myget.org/F/icnaming/npm/
npm install
npx icdev init-identity
npx ts-node -r tsconfig-paths/register scripts/index.ts update-local-config
