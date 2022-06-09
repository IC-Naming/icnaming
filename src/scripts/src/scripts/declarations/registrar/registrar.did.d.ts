import type { Principal } from '@dfinity/principal';
export type BooleanActorResponse = { 'Ok' : boolean } |
  { 'Err' : ErrorInfo };
export interface ErrorInfo { 'code' : number, 'message' : string }
export type GetAllDetailsActorResponse = { 'Ok' : Array<RegistrationDetails> } |
  { 'Err' : ErrorInfo };
export type GetDetailsActorResponse = { 'Ok' : RegistrationDetails } |
  { 'Err' : ErrorInfo };
export type GetNameExpiresActorResponse = { 'Ok' : bigint } |
  { 'Err' : ErrorInfo };
export interface GetNameOrderResponse {
  'status' : NameOrderStatus,
  'payment_memo' : PaymentMemo,
  'name' : string,
  'price_icp_in_e8s' : bigint,
  'payment_account_id' : Array<number>,
  'quota_type' : QuotaType,
  'payment_id' : bigint,
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
export type GetQuotaActorResponse = { 'Ok' : number } |
  { 'Err' : ErrorInfo };
export type GetStatsActorResponse = { 'Ok' : Stats } |
  { 'Err' : ErrorInfo };
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
export type NameOrderStatus = { 'New' : null } |
  { 'WaitingToRefund' : null } |
  { 'Done' : null } |
  { 'Canceled' : null };
export type PaymentMemo = { 'ICP' : bigint };
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
export type SubmitOrderActorResponse = { 'Ok' : SubmitOrderResponse } |
  { 'Err' : ErrorInfo };
export interface SubmitOrderRequest { 'name' : string, 'years' : number }
export interface SubmitOrderResponse { 'order' : GetNameOrderResponse }
export interface TransferFromQuotaRequest {
  'to' : Principal,
  'diff' : number,
  'from' : Principal,
  'quota_type' : QuotaType,
}
export interface _SERVICE {
  'add_quota' : (arg_0: Principal, arg_1: QuotaType, arg_2: number) => Promise<
      BooleanActorResponse
    >,
  'approve' : (arg_0: string, arg_1: Principal) => Promise<
      BooleanActorResponse
    >,
  'available' : (arg_0: string) => Promise<BooleanActorResponse>,
  'cancel_order' : () => Promise<BooleanActorResponse>,
  'confirm_pay_order' : (arg_0: bigint) => Promise<BooleanActorResponse>,
  'export_state' : () => Promise<StateExportResponse>,
  'get_all_details' : (arg_0: GetPageInput) => Promise<
      GetAllDetailsActorResponse
    >,
  'get_details' : (arg_0: string) => Promise<GetDetailsActorResponse>,
  'get_last_registrations' : () => Promise<GetAllDetailsActorResponse>,
  'get_name_expires' : (arg_0: string) => Promise<GetNameExpiresActorResponse>,
  'get_names' : (arg_0: Principal, arg_1: GetPageInput) => Promise<
      GetNamesActorResponse
    >,
  'get_owner' : (arg_0: string) => Promise<GetOwnerActorResponse>,
  'get_pending_order' : () => Promise<GetPendingOrderActorResponse>,
  'get_price_table' : () => Promise<GetPriceTableResponse>,
  'get_quota' : (arg_0: Principal, arg_1: QuotaType) => Promise<
      GetQuotaActorResponse
    >,
  'get_stats' : () => Promise<GetStatsActorResponse>,
  'import_quota' : (arg_0: ImportQuotaRequest) => Promise<ImportQuotaResponse>,
  'load_state' : (arg_0: StateExportData) => Promise<BooleanActorResponse>,
  'refund_order' : () => Promise<BooleanActorResponse>,
  'register_for' : (arg_0: string, arg_1: Principal, arg_2: bigint) => Promise<
      BooleanActorResponse
    >,
  'register_from_gateway' : (arg_0: string, arg_1: Principal) => Promise<
      BooleanActorResponse
    >,
  'register_with_quota' : (arg_0: string, arg_1: QuotaType) => Promise<
      BooleanActorResponse
    >,
  'run_tasks' : () => Promise<BooleanActorResponse>,
  'set_maintaining_time' : (arg_0: bigint) => Promise<BooleanActorResponse>,
  'sub_quota' : (arg_0: Principal, arg_1: QuotaType, arg_2: number) => Promise<
      BooleanActorResponse
    >,
  'submit_order' : (arg_0: SubmitOrderRequest) => Promise<
      SubmitOrderActorResponse
    >,
  'transfer' : (arg_0: string, arg_1: Principal) => Promise<
      BooleanActorResponse
    >,
  'transfer_by_admin' : (arg_0: string, arg_1: Principal) => Promise<
      BooleanActorResponse
    >,
  'transfer_from' : (arg_0: string) => Promise<BooleanActorResponse>,
  'transfer_from_quota' : (arg_0: TransferFromQuotaRequest) => Promise<
      BooleanActorResponse
    >,
  'transfer_quota' : (
      arg_0: Principal,
      arg_1: QuotaType,
      arg_2: number,
    ) => Promise<BooleanActorResponse>,
  'unlock_names' : (arg_0: Array<string>) => Promise<BooleanActorResponse>,
}
