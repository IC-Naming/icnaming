import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

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
export interface ErrorInfo { 'code' : number, 'message' : string }
export type GetAllDetailsActorResponse = { 'Ok' : Array<RegistrationDetails> } |
  { 'Err' : ErrorInfo };
export type GetDetailsActorResponse = { 'Ok' : RegistrationDetails } |
  { 'Err' : ErrorInfo };
export type GetNameExpiresActorResponse = { 'Ok' : bigint } |
  { 'Err' : ErrorInfo };
export interface GetNameOrderResponse {
  'status' : NameOrderStatus,
  'name' : string,
  'created_at' : bigint,
  'price_icp_in_e8s' : bigint,
  'created_user' : Principal,
  'years' : number,
}
export type GetNamesActorResponse = { 'Ok' : GetPageOutput } |
  { 'Err' : ErrorInfo };
export type GetOwnerActorResponse = { 'Ok' : Principal } |
  { 'Err' : ErrorInfo };
export interface GetPageInput { 'offset' : bigint, 'limit' : bigint }
export interface GetPageOutput { 'items' : Array<RegistrationDto> }
export type GetPendingOrderActorResponse = {
    'Ok' : [] | [GetNameOrderResponse]
  } |
  { 'Err' : ErrorInfo };
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
export type NameOrderStatus = { 'New' : null } |
  { 'WaitingToRefund' : null } |
  { 'Done' : null } |
  { 'Canceled' : null };
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
  'name_order_cancelled_count' : bigint,
  'user_quota_count' : Array<[string, bigint]>,
  'name_order_placed_count' : bigint,
  'name_order_paid_count' : bigint,
  'user_name_order_count_by_status' : Array<[string, bigint]>,
  'last_timestamp_seconds_xdr_permyriad_per_icp' : bigint,
  'name_lock_count' : bigint,
  'payment_version' : bigint,
  'user_quota_order_count' : Array<[string, bigint]>,
  'registration_count' : bigint,
}
export type StreamingStrategy = { 'Callback' : CallbackStrategy };
export type SubmitOrderActorResponse = { 'Ok' : SubmitOrderResponse } |
  { 'Err' : ErrorInfo };
export interface SubmitOrderRequest { 'name' : string, 'years' : number }
export interface SubmitOrderResponse { 'order' : GetNameOrderResponse }
export interface Token {
  'key' : string,
  'sha256' : [] | [Array<number>],
  'index' : bigint,
  'content_encoding' : string,
}
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
export interface _SERVICE {
  'add_quota' : ActorMethod<
    [Principal, QuotaType, number],
    BooleanActorResponse,
  >,
  'approve' : ActorMethod<[string, Principal], BooleanActorResponse>,
  'available' : ActorMethod<[string], BooleanActorResponse>,
  'batch_transfer_quota' : ActorMethod<
    [BatchTransferRequest],
    BooleanActorResponse,
  >,
  'cancel_order' : ActorMethod<[], BooleanActorResponse>,
  'export_state' : ActorMethod<[], StateExportResponse>,
  'get_all_details' : ActorMethod<[GetPageInput], GetAllDetailsActorResponse>,
  'get_details' : ActorMethod<[string], GetDetailsActorResponse>,
  'get_last_registrations' : ActorMethod<[], GetAllDetailsActorResponse>,
  'get_name_expires' : ActorMethod<[string], GetNameExpiresActorResponse>,
  'get_names' : ActorMethod<[Principal, GetPageInput], GetNamesActorResponse>,
  'get_owner' : ActorMethod<[string], GetOwnerActorResponse>,
  'get_pending_order' : ActorMethod<[], GetPendingOrderActorResponse>,
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
  'pay_my_order' : ActorMethod<[], BooleanActorResponse>,
  'reclaim_name' : ActorMethod<[string], BooleanActorResponse>,
  'register_for' : ActorMethod<
    [string, Principal, bigint],
    BooleanActorResponse,
  >,
  'register_from_gateway' : ActorMethod<
    [string, Principal],
    BooleanActorResponse,
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
  'submit_order' : ActorMethod<[SubmitOrderRequest], SubmitOrderActorResponse>,
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
