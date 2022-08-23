import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface AllowanceRequest {
  'token' : string,
  'owner' : User,
  'spender' : Principal,
}
export interface ApproveRequest {
  'token' : string,
  'subaccount' : [] | [Array<number>],
  'allowance' : bigint,
  'spender' : Principal,
}
export interface BatchAddQuotaRequest { 'items' : Array<ImportQuotaItem> }
export interface BatchTransferRequest { 'items' : Array<TransferQuotaDetails> }
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
export type CommonError = { 'InvalidToken' : string } |
  { 'Other' : string };
export type EXTAllowanceActorResponse = { 'Ok' : bigint } |
  { 'Err' : CommonError };
export type EXTBearerActorResponse = { 'Ok' : string } |
  { 'Err' : CommonError };
export type EXTMetadataActorResponse = { 'Ok' : Metadata } |
  { 'Err' : CommonError };
export type EXTSupplyActorResponse = { 'Ok' : bigint } |
  { 'Err' : CommonError };
export type EXTTransferResponse = { 'Ok' : bigint } |
  { 'Err' : TransferError };
export interface ErrorInfo { 'code' : number, 'message' : string }
export interface Fungible {
  'decimals' : string,
  'metadata' : [] | [Array<number>],
  'name' : User,
  'symbol' : Principal,
}
export type GetAllDetailsActorResponse = { 'Ok' : Array<RegistrationDetails> } |
  { 'Err' : ErrorInfo };
export type GetDetailsActorResponse = { 'Ok' : RegistrationDetails } |
  { 'Err' : ErrorInfo };
export type GetNameExpiresActorResponse = { 'Ok' : bigint } |
  { 'Err' : ErrorInfo };
export type GetNameStatueActorResponse = { 'Ok' : NameStatus } |
  { 'Err' : ErrorInfo };
export type GetNamesActorResponse = { 'Ok' : GetPageOutput } |
  { 'Err' : ErrorInfo };
export type GetNamesCountActorResponse = { 'Ok' : number } |
  { 'Err' : ErrorInfo };
export type GetOwnerActorResponse = { 'Ok' : Principal } |
  { 'Err' : ErrorInfo };
export interface GetPageInput { 'offset' : bigint, 'limit' : bigint }
export interface GetPageOutput { 'items' : Array<RegistrationDto> }
export type GetPriceTableResponse = { 'Ok' : PriceTable } |
  { 'Err' : ErrorInfo };
export type GetPublicResolverActorResponse = { 'Ok' : string } |
  { 'Err' : ErrorInfo };
export type GetQuotaActorResponse = { 'Ok' : number } |
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
export interface ImportNameRegistrationItem {
  'owner' : Principal,
  'name' : string,
  'years' : number,
}
export interface ImportNameRegistrationRequest {
  'items' : Array<ImportNameRegistrationItem>,
}
export interface ImportQuotaItem {
  'owner' : Principal,
  'diff' : number,
  'quota_type' : string,
}
export interface ImportQuotaRequest {
  'hash' : Array<number>,
  'items' : Array<ImportQuotaItem>,
}
export type ImportQuotaResponse = { 'Ok' : ImportQuotaStatus } |
  { 'Err' : ErrorInfo };
export type ImportQuotaStatus = { 'Ok' : null } |
  { 'AlreadyExists' : null };
export interface InitArgs {
  'dev_named_canister_ids' : Array<[CanisterNames, Principal]>,
}
export type Metadata = { 'fungible' : Fungible } |
  { 'nonfungible' : NonFungible };
export interface NameStatus {
  'kept' : boolean,
  'available' : boolean,
  'details' : [] | [RegistrationDetails],
  'registered' : boolean,
}
export interface NonFungible { 'metadata' : [] | [Array<number>] }
export interface PriceTable {
  'icp_xdr_conversion_rate' : bigint,
  'items' : Array<PriceTableItem>,
}
export interface PriceTableItem {
  'len' : number,
  'price_in_icp_e8s' : bigint,
  'price_in_xdr_permyriad' : bigint,
}
export type QuotaType = { 'LenEq' : number } |
  { 'LenGte' : number };
export interface RegisterNameWithPaymentRequest {
  'name' : string,
  'approve_amount' : bigint,
  'years' : number,
}
export interface RegistrationDetails {
  'owner' : Principal,
  'name' : string,
  'created_at' : bigint,
  'expired_at' : bigint,
}
export interface RegistrationDto {
  'name' : string,
  'created_at' : bigint,
  'expired_at' : bigint,
}
export interface RenewNameRequest {
  'name' : string,
  'approve_amount' : bigint,
  'years' : number,
}
export interface StateExportData { 'state_data' : Array<number> }
export type StateExportResponse = { 'Ok' : StateExportData } |
  { 'Err' : ErrorInfo };
export interface Stats {
  'user_count' : bigint,
  'new_registered_name_count' : bigint,
  'cycles_balance' : bigint,
  'last_xdr_permyriad_per_icp' : bigint,
  'user_quota_count' : Array<[string, bigint]>,
  'name_order_paid_count' : bigint,
  'last_timestamp_seconds_xdr_permyriad_per_icp' : bigint,
  'name_lock_count' : bigint,
  'registration_count' : bigint,
}
export type StreamingStrategy = { 'Callback' : CallbackStrategy };
export interface Token {
  'key' : string,
  'sha256' : [] | [Array<number>],
  'index' : bigint,
  'content_encoding' : string,
}
export type TransferError = { 'CannotNotify' : string } |
  { 'InsufficientBalance' : null } |
  { 'InvalidToken' : string } |
  { 'Rejected' : null } |
  { 'Unauthorized' : string } |
  { 'Other' : string };
export interface TransferFromQuotaRequest {
  'to' : Principal,
  'diff' : number,
  'from' : Principal,
  'quota_type' : QuotaType,
}
export interface TransferQuotaDetails {
  'to' : Principal,
  'diff' : number,
  'quota_type' : QuotaType,
}
export interface TransferRequest {
  'to' : User,
  'token' : string,
  'notify' : boolean,
  'from' : User,
  'memo' : Array<number>,
  'subaccount' : [] | [Array<number>],
  'amount' : bigint,
}
export type User = { 'principal' : Principal } |
  { 'address' : string };
export interface _SERVICE {
  'add_quota' : ActorMethod<
    [Principal, QuotaType, number],
    BooleanActorResponse,
  >,
  'approve' : ActorMethod<[string, Principal], BooleanActorResponse>,
  'available' : ActorMethod<[string], BooleanActorResponse>,
  'batch_add_quota' : ActorMethod<[BatchAddQuotaRequest], BooleanActorResponse>,
  'batch_transfer_quota' : ActorMethod<
    [BatchTransferRequest],
    BooleanActorResponse,
  >,
  'export_state' : ActorMethod<[], StateExportResponse>,
  'ext_allowance' : ActorMethod<[AllowanceRequest], EXTAllowanceActorResponse>,
  'ext_approve' : ActorMethod<[ApproveRequest], undefined>,
  'ext_bearer' : ActorMethod<[string], EXTBearerActorResponse>,
  'ext_getMinter' : ActorMethod<[], Principal>,
  'ext_getRegistry' : ActorMethod<[], Array<[number, string]>>,
  'ext_getTokens' : ActorMethod<[], Array<[number, Metadata]>>,
  'ext_metadata' : ActorMethod<[string], EXTMetadataActorResponse>,
  'ext_supply' : ActorMethod<[], EXTSupplyActorResponse>,
  'ext_transfer' : ActorMethod<[TransferRequest], EXTTransferResponse>,
  'get_all_details' : ActorMethod<[GetPageInput], GetAllDetailsActorResponse>,
  'get_details' : ActorMethod<[string], GetDetailsActorResponse>,
  'get_last_registrations' : ActorMethod<[], GetAllDetailsActorResponse>,
  'get_name_expires' : ActorMethod<[string], GetNameExpiresActorResponse>,
  'get_name_status' : ActorMethod<[string], GetNameStatueActorResponse>,
  'get_names' : ActorMethod<[Principal, GetPageInput], GetNamesActorResponse>,
  'get_names_count' : ActorMethod<[Principal], GetNamesCountActorResponse>,
  'get_owner' : ActorMethod<[string], GetOwnerActorResponse>,
  'get_price_table' : ActorMethod<[], GetPriceTableResponse>,
  'get_public_resolver' : ActorMethod<[], GetPublicResolverActorResponse>,
  'get_quota' : ActorMethod<[Principal, QuotaType], GetQuotaActorResponse>,
  'get_stats' : ActorMethod<[], GetStatsResponse>,
  'get_wasm_info' : ActorMethod<[], Array<[string, string]>>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'import_quota' : ActorMethod<[ImportQuotaRequest], ImportQuotaResponse>,
  'import_registrations' : ActorMethod<
    [ImportNameRegistrationRequest],
    BooleanActorResponse,
  >,
  'load_state' : ActorMethod<[StateExportData], BooleanActorResponse>,
  'reclaim_name' : ActorMethod<[string], BooleanActorResponse>,
  'register_for' : ActorMethod<
    [string, Principal, bigint],
    BooleanActorResponse,
  >,
  'register_from_gateway' : ActorMethod<
    [string, Principal],
    BooleanActorResponse,
  >,
  'register_with_payment' : ActorMethod<
    [RegisterNameWithPaymentRequest],
    GetDetailsActorResponse,
  >,
  'register_with_quota' : ActorMethod<
    [string, QuotaType],
    BooleanActorResponse,
  >,
  'renew_name' : ActorMethod<[RenewNameRequest], BooleanActorResponse>,
  'run_tasks' : ActorMethod<[], BooleanActorResponse>,
  'sub_quota' : ActorMethod<
    [Principal, QuotaType, number],
    BooleanActorResponse,
  >,
  'transfer' : ActorMethod<[string, Principal], BooleanActorResponse>,
  'transfer_by_admin' : ActorMethod<[string, Principal], BooleanActorResponse>,
  'transfer_from' : ActorMethod<[string], BooleanActorResponse>,
  'transfer_from_quota' : ActorMethod<
    [TransferFromQuotaRequest],
    BooleanActorResponse,
  >,
  'transfer_quota' : ActorMethod<
    [Principal, QuotaType, number],
    BooleanActorResponse,
  >,
  'unlock_names' : ActorMethod<[Array<string>], BooleanActorResponse>,
}
