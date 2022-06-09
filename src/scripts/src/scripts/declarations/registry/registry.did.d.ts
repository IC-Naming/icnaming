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
  'export_state' : () => Promise<StateExportResponse>,
  'get_controlled_names' : (arg_0: Principal, arg_1: GetPageInput) => Promise<
      GetControlledNamesResponse
    >,
  'get_details' : (arg_0: string) => Promise<GetDetailsResponse>,
  'get_owner' : (arg_0: string) => Promise<GetOwnerResponse>,
  'get_resolver' : (arg_0: string) => Promise<GetOwnerResponse>,
  'get_stats' : () => Promise<GetStatsResponse>,
  'get_ttl' : (arg_0: string) => Promise<GetTtlResponse>,
  'get_users' : (arg_0: string) => Promise<GetUsersResponse>,
  'get_wasm_info' : () => Promise<Array<[string, string]>>,
  'http_request' : (arg_0: HttpRequest) => Promise<HttpResponse>,
  'load_state' : (arg_0: StateExportData) => Promise<BooleanActorResponse>,
  'reclaim_name' : (
      arg_0: string,
      arg_1: Principal,
      arg_2: Principal,
    ) => Promise<BooleanActorResponse>,
  'set_approval' : (arg_0: string, arg_1: Principal, arg_2: boolean) => Promise<
      BooleanActorResponse
    >,
  'set_owner' : (arg_0: string, arg_1: Principal) => Promise<
      BooleanActorResponse
    >,
  'set_record' : (arg_0: string, arg_1: bigint, arg_2: Principal) => Promise<
      BooleanActorResponse
    >,
  'set_resolver' : (arg_0: string, arg_1: Principal) => Promise<
      BooleanActorResponse
    >,
  'set_subdomain_owner' : (
      arg_0: string,
      arg_1: string,
      arg_2: Principal,
      arg_3: bigint,
      arg_4: Principal,
    ) => Promise<GetDetailsResponse>,
  'transfer' : (arg_0: string, arg_1: Principal, arg_2: Principal) => Promise<
      BooleanActorResponse
    >,
}
