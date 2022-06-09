import type { Principal } from '@dfinity/principal';
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
  'add_favorite' : (arg_0: string) => Promise<BooleanActorResponse>,
  'export_state' : () => Promise<StateExportResponse>,
  'get_favorites' : () => Promise<GetFavoritesResponse>,
  'get_stats' : () => Promise<GetStatsResponse>,
  'get_wasm_info' : () => Promise<Array<[string, string]>>,
  'http_request' : (arg_0: HttpRequest) => Promise<HttpResponse>,
  'load_state' : (arg_0: StateExportData) => Promise<BooleanActorResponse>,
  'remove_favorite' : (arg_0: string) => Promise<BooleanActorResponse>,
}
