import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

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
export type GetFavoritesResponse = { 'Ok' : Array<string> } |
  { 'Err' : ErrorInfo };
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
export interface InitArgs {
  'dev_named_canister_ids' : Array<[CanisterNames, Principal]>,
}
export interface StateExportData { 'state_data' : Array<number> }
export type StateExportResponse = { 'Ok' : StateExportData } |
  { 'Err' : ErrorInfo };
export interface Stats {
  'user_count' : bigint,
  'cycles_balance' : bigint,
  'favorite_count' : bigint,
}
export type StreamingStrategy = { 'Callback' : CallbackStrategy };
export interface Token {
  'key' : string,
  'sha256' : [] | [Array<number>],
  'index' : bigint,
  'content_encoding' : string,
}
export interface _SERVICE {
  'add_favorite' : ActorMethod<[string], BooleanActorResponse>,
  'export_state' : ActorMethod<[], StateExportResponse>,
  'get_favorites' : ActorMethod<[], GetFavoritesResponse>,
  'get_stats' : ActorMethod<[], GetStatsResponse>,
  'get_wasm_info' : ActorMethod<[], Array<[string, string]>>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'load_state' : ActorMethod<[StateExportData], BooleanActorResponse>,
  'remove_favorite' : ActorMethod<[string], BooleanActorResponse>,
}
