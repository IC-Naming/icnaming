import type { Principal } from '@dfinity/principal';
export type AssignNameResponse = { 'Ok' : AssignNameResult } |
  { 'Err' : ErrorInfo };
export type AssignNameResult = { 'Ok' : null } |
  { 'AlreadyAssigned' : null } |
  { 'FailFromRegistrar' : null };
export type BooleanActorResponse = { 'Ok' : boolean } |
  { 'Err' : ErrorInfo };
export interface ErrorInfo { 'code' : number, 'message' : string }
export type GetStatsActorResponse = { 'Ok' : Stats } |
  { 'Err' : ErrorInfo };
export type ImportQuotaResponse = { 'Ok' : ImportQuotaResult } |
  { 'Err' : ErrorInfo };
export type ImportQuotaResult = { 'Ok' : null } |
  { 'AlreadyExists' : null } |
  { 'InvalidRequest' : null };
export interface StateExportData { 'state_data' : Array<number> }
export type StateExportResponse = { 'Ok' : StateExportData } |
  { 'Err' : ErrorInfo };
export interface Stats {
  'name_assignments_count' : bigint,
  'cycles_balance' : bigint,
  'imported_file_hashes_count' : bigint,
  'acceptable_file_hashes_count' : bigint,
}
export interface _SERVICE {
  'assign_name' : (arg_0: string, arg_1: Principal) => Promise<
      AssignNameResponse
    >,
  'export_state' : () => Promise<StateExportResponse>,
  'get_stats' : () => Promise<GetStatsActorResponse>,
  'import_quota' : (arg_0: Array<number>) => Promise<ImportQuotaResponse>,
  'load_state' : (arg_0: StateExportData) => Promise<BooleanActorResponse>,
}
