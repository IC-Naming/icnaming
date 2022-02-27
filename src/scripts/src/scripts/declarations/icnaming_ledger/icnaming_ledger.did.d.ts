import type { Principal } from '@dfinity/principal';
export type AccountIdentifier = Array<number>;
export interface AddPaymentRequest {
  'created_remark' : string,
  'amount' : ICPTs,
}
export interface AddPaymentResponse {
  'memo' : Memo,
  'payment_account_id' : AccountIdentifier,
  'payment_id' : PaymentId,
}
export type BlockHeight = bigint;
export type CanisterId = Principal;
export interface ICPTs { 'e8s' : bigint }
export type Memo = bigint;
export type NeuronId = bigint;
export type PaymentId = bigint;
export interface RefundPaymentRequest { 'payment_id' : PaymentId }
export type RefundPaymentResponse = { 'Refunding' : null } |
  { 'Refunded' : { 'refunded_amount' : ICPTs } } |
  { 'PaymentNotFound' : null } |
  { 'RefundFailed' : null };
export interface Stats {
  'latest_transaction_block_height' : BlockHeight,
  'seconds_since_last_ledger_sync' : bigint,
  'sub_accounts_count' : bigint,
  'neurons_topped_up_count' : bigint,
  'transactions_to_process_queue_length' : number,
  'neurons_created_count' : bigint,
  'hardware_wallet_accounts_count' : bigint,
  'accounts_count' : bigint,
  'earliest_transaction_block_height' : BlockHeight,
  'transactions_count' : bigint,
  'block_height_synced_up_to' : [] | [bigint],
  'latest_transaction_timestamp_nanos' : bigint,
  'earliest_transaction_timestamp_nanos' : bigint,
}
export type SubAccount = Array<number>;
export interface SyncICPPaymentRequest { 'block_height' : BlockHeight }
export interface SyncICPPaymentResponse {
  'verify_payment_response' : [] | [VerifyPaymentResponse],
  'payment_id' : [] | [PaymentId],
}
export interface Timestamp { 'timestamp_nanos' : bigint }
export interface VerifyPaymentRequest { 'payment_id' : PaymentId }
export type VerifyPaymentResponse = { 'Paid' : { 'paid_at' : Timestamp } } |
  { 'PaymentNotFound' : null } |
  { 'NeedMore' : { 'received_amount' : ICPTs, 'amount' : ICPTs } };
export interface _SERVICE {
  'add_payment' : (arg_0: AddPaymentRequest) => Promise<AddPaymentResponse>,
  'add_stable_asset' : (arg_0: Array<number>) => Promise<undefined>,
  'get_stats' : () => Promise<Stats>,
  'refund_payment' : (arg_0: RefundPaymentRequest) => Promise<
      RefundPaymentResponse
    >,
  'sync_icp_payment' : (arg_0: SyncICPPaymentRequest) => Promise<
      SyncICPPaymentResponse
    >,
  'verify_payment' : (arg_0: VerifyPaymentRequest) => Promise<
      VerifyPaymentResponse
    >,
}
