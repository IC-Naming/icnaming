import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type BatchGetReverseResolvePrincipalResponse = {
    'Ok' : Array<[Principal, [] | [string]]>
  } |
  { 'Err' : ErrorInfo };
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
export type GetRecordValueResponse = { 'Ok' : Array<[string, string]> } |
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
export type ReverseResolvePrincipalResponse = { 'Ok' : [] | [string] } |
  { 'Err' : ErrorInfo };
export interface StateExportData { 'state_data' : Array<number> }
export type StateExportResponse = { 'Ok' : StateExportData } |
  { 'Err' : ErrorInfo };
export interface Stats { 'cycles_balance' : bigint, 'resolver_count' : bigint }
export type StreamingStrategy = { 'Callback' : CallbackStrategy };
export interface Token {
  'key' : string,
  'sha256' : [] | [Array<number>],
  'index' : bigint,
  'content_encoding' : string,
}
export interface _SERVICE {
  'batch_get_reverse_resolve_principal' : ActorMethod<
    [Array<Principal>],
    BatchGetReverseResolvePrincipalResponse,
  >,
  'ensure_resolver_created' : ActorMethod<[string], BooleanActorResponse>,
  'export_state' : ActorMethod<[], StateExportResponse>,
  'get_record_value' : ActorMethod<[string], GetRecordValueResponse>,
  'get_stats' : ActorMethod<[], GetStatsResponse>,
  'get_wasm_info' : ActorMethod<[], Array<[string, string]>>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'load_state' : ActorMethod<[StateExportData], BooleanActorResponse>,
  'remove_resolvers' : ActorMethod<[Array<string>], BooleanActorResponse>,
  'reverse_resolve_principal' : ActorMethod<
    [Principal],
    ReverseResolvePrincipalResponse,
  >,
  'set_record_value' : ActorMethod<
    [string, Array<[string, string]>],
    BooleanActorResponse,
  >,
}
