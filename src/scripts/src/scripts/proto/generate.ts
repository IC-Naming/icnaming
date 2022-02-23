import { exec } from "shelljs";

const generateTSCode = (): void => {
  exec(
    "pbjs -t static-module -w es6 ic_base_types.proto | pbts -o ic_base_types.ts -"
  ); exec(
    "pbjs -t static-module -w es6 ic_base_types.proto | pbts -o ic_base_types.ts -"
  );
};

generateTSCode();