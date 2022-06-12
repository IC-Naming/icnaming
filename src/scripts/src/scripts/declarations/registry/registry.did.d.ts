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
export type GetControlledNamesResponse = { 'Ok' : GetPageOutput } |
  { 'Err' : ErrorInfo };
export type GetDetailsResponse = { 'Ok' : RegistryDto } |
  { 'Err' : ErrorInfo };
export type GetOwnerResponse = { 'Ok' : Principal } |
  { 'Err' : ErrorInfo };
export interface GetPageInput { 'offset' : bigint, 'limit' : bigint }
export interface GetPageOutput { 'items' : Array<string> }
export type GetStatsResponse = { 'Ok' : Stats } |
  { 'Err' : ErrorInfo };
export type GetTtlResponse = { 'Ok' : bigint } |
  { 'Err' : ErrorInfo };
export type GetUsersResponse = { 'Ok' : RegistryUsers } |
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
export interface RegistryDto {
  'ttl' : bigint,
  'resolver' : Principal,
  'owner' : Principal,
  'name' : string,
}
export interface RegistryUsers {
  'owner' : Principal,
  'operators' : Array<Principal>,
}
export interface StateExportData { 'state_data' : Array<number> }
export type StateExportResponse = { 'Ok' : StateExportData } |
  { 'Err' : ErrorInfo };
export interface Stats { 'cycles_balance' : bigint, 'registry_count' : bigint }
export type StreamingStrategy = { 'Callback' : CallbackStrategy };
export interface Token {
  'key' : string,
  'sha256' : [] | [Array<number>],
  'index' : bigint,
  'content_encoding' : string,
}
export interface _SERVICE {
  'export_state' : ActorMethod<[], StateExportResponse>,
  'get_controlled_names' : ActorMethod<
    [Principal, GetPageInput],
    GetControlledNamesResponse,
  >,
  'get_details' : ActorMethod<[string], GetDetailsResponse>,
  'get_owner' : ActorMethod<[string], GetOwnerResponse>,
  'get_resolver' : ActorMethod<[string], GetOwnerResponse>,
  'get_stats' : ActorMethod<[], GetStatsResponse>,
  'get_ttl' : ActorMethod<[string], GetTtlResponse>,
  'get_users' : ActorMethod<[string], GetUsersResponse>,
  'get_wasm_info' : ActorMethod<[], Array<[string, string]>>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'load_state' : ActorMethod<[StateExportData], BooleanActorResponse>,
  'reclaim_name' : ActorMethod<
    [string, Principal, Principal],
    BooleanActorResponse,
  >,
  'set_approval' : ActorMethod<
    [string, Principal, boolean],
    BooleanActorResponse,
  >,
  'set_owner' : ActorMethod<[string, Principal], BooleanActorResponse>,
  'set_record' : ActorMethod<[string, bigint, Principal], BooleanActorResponse>,
  'set_resolver' : ActorMethod<[string, Principal], BooleanActorResponse>,
  'set_subdomain_owner' : ActorMethod<
    [string, string, Principal, bigint, Principal],
    GetDetailsResponse,
  >,
  'transfer' : ActorMethod<
    [string, Principal, Principal],
    BooleanActorResponse,
  >,
}
