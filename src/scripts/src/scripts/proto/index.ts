import { load, loadSync } from "protobufjs";
import { resolve } from "path";

export const loadProto = () => {
  return loadSync([
    resolve(__dirname, "./ic_base_types.proto"),
    resolve(__dirname, "./ic_nns_common.proto"),
    resolve(__dirname, "./ic_ledger.proto"),
    resolve(__dirname, "./governance.proto"),
  ]);
};
