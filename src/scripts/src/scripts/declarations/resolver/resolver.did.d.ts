import type { Principal } from '@dfinity/principal';
export type BooleanActorResponse = { 'Ok' : boolean } |
  { 'Err' : ErrorInfo };
export interface ErrorInfo { 'code' : number, 'message' : string }
export type GetRecordValueResponse = { 'Ok' : Array<[string, string]> } |
  { 'Err' : ErrorInfo };
export type GetStatsResponse = { 'Ok' : Stats } |
  { 'Err' : ErrorInfo };
export interface StateExportData { 'state_data' : Array<number> }
export type StateExportResponse = { 'Ok' : StateExportData } |
  { 'Err' : ErrorInfo };
export interface Stats { 'cycles_balance' : bigint, 'resolver_count' : bigint }
export interface _SERVICE {
  'ensure_resolver_created' : (arg_0: string) => Promise<BooleanActorResponse>,
  'export_state' : () => Promise<StateExportResponse>,
  'get_record_value' : (arg_0: string) => Promise<GetRecordValueResponse>,
  'get_stats' : () => Promise<GetStatsResponse>,
  'load_state' : (arg_0: StateExportData) => Promise<BooleanActorResponse>,
  'remove_resolvers' : (arg_0: Array<string>) => Promise<BooleanActorResponse>,
  'set_record_value' : (
      arg_0: string,
      arg_1: Array<[string, string]>,
    ) => Promise<BooleanActorResponse>,
}
