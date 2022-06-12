import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type AssignNameResponse = { 'Ok' : AssignNameResult } |
  { 'Err' : ErrorInfo };
export type AssignNameResult = { 'Ok' : null } |
  { 'AlreadyAssigned' : null } |
  { 'FailFromRegistrar' : null };
export type BooleanActorResponse = { 'Ok' : boolean } |
  { 'Err' : ErrorInfo };
export interface CallbackStrategy {
  'token' : Token,
  'callback' : [Principal, string],
}
export type CanisterNames = { 'NamingMarketplace' : null } |
  { 'RegistrarControlGateway' : null } |
  { 'DICP' : null } |
  { 'CyclesMinting' : null } |
  { 'Registrar' : null } |
  { 'MysteryBox' : null } |
  { 'Registry' : null } |
  { 'Ledger' : null } |
  { 'Favorites' : null } |
  { 'Resolver' : null };
export interface ErrorInfo { 'code' : number, 'message' : string }
export type GetStatsResponse = { 'Ok' : Stats } |
  { 'Err' : ErrorInfo };
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Array<number>,
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'body' : Array<number>,
  'headers' : Array<[string, string]>,
  'streaming_strategy' : [] | [StreamingStrategy],
  'status_code' : number,
}
export type ImportQuotaResponse = { 'Ok' : ImportQuotaResult } |
  { 'Err' : ErrorInfo };
export type ImportQuotaResult = { 'Ok' : null } |
  { 'AlreadyExists' : null } |
  { 'InvalidRequest' : null };
export interface InitArgs {
  'dev_named_canister_ids' : Array<[CanisterNames, Principal]>,
}
export interface StateExportData { 'state_data' : Array<number> }
export type StateExportResponse = { 'Ok' : StateExportData } |
  { 'Err' : ErrorInfo };
export interface Stats {
  'name_assignments_count' : bigint,
  'cycles_balance' : bigint,
  'imported_file_hashes_count' : bigint,
  'acceptable_file_hashes_count' : bigint,
}
export type StreamingStrategy = { 'Callback' : CallbackStrategy };
export interface Token {
  'key' : string,
  'sha256' : [] | [Array<number>],
  'index' : bigint,
  'content_encoding' : string,
}
export interface _SERVICE {
  'assign_name' : ActorMethod<[string, Principal], AssignNameResponse>,
  'export_state' : ActorMethod<[], StateExportResponse>,
  'get_stats' : ActorMethod<[], GetStatsResponse>,
  'get_wasm_info' : ActorMethod<[], Array<[string, string]>>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'import_quota' : ActorMethod<[Array<number>], ImportQuotaResponse>,
  'load_state' : ActorMethod<[StateExportData], BooleanActorResponse>,
}
