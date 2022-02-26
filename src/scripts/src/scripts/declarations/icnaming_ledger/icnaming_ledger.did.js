export const idlFactory = ({ IDL }) => {
  const ICPTs = IDL.Record({ 'e8s' : IDL.Nat64 });
  const AddPaymentRequest = IDL.Record({
    'created_remark' : IDL.Text,
    'amount' : ICPTs,
  });
  const Memo = IDL.Nat64;
  const AccountIdentifier = IDL.Vec(IDL.Nat8);
  const PaymentId = IDL.Nat64;
  const AddPaymentResponse = IDL.Record({
    'memo' : Memo,
    'payment_account_id' : AccountIdentifier,
    'payment_id' : PaymentId,
  });
  const BlockHeight = IDL.Nat64;
  const Stats = IDL.Record({
    'latest_transaction_block_height' : BlockHeight,
    'seconds_since_last_ledger_sync' : IDL.Nat64,
    'sub_accounts_count' : IDL.Nat64,
    'neurons_topped_up_count' : IDL.Nat64,
    'transactions_to_process_queue_length' : IDL.Nat32,
    'neurons_created_count' : IDL.Nat64,
    'hardware_wallet_accounts_count' : IDL.Nat64,
    'accounts_count' : IDL.Nat64,
    'earliest_transaction_block_height' : BlockHeight,
    'transactions_count' : IDL.Nat64,
    'block_height_synced_up_to' : IDL.Opt(IDL.Nat64),
    'latest_transaction_timestamp_nanos' : IDL.Nat64,
    'earliest_transaction_timestamp_nanos' : IDL.Nat64,
  });
  const RefundPaymentRequest = IDL.Record({ 'payment_id' : PaymentId });
  const RefundPaymentResponse = IDL.Variant({
    'Refunding' : IDL.Null,
    'Refunded' : IDL.Record({ 'refunded_amount' : ICPTs }),
    'PaymentNotFound' : IDL.Null,
    'RefundFailed' : IDL.Null,
  });
  const SyncICPPaymentRequest = IDL.Record({ 'block_height' : BlockHeight });
  const Timestamp = IDL.Record({ 'timestamp_nanos' : IDL.Nat64 });
  const VerifyPaymentResponse = IDL.Variant({
    'Paid' : IDL.Record({ 'paid_at' : Timestamp }),
    'PaymentNotFound' : IDL.Null,
    'NeedMore' : IDL.Record({ 'received_amount' : ICPTs, 'amount' : ICPTs }),
  });
  const VerifyPaymentRequest = IDL.Record({ 'payment_id' : PaymentId });
  return IDL.Service({
    'add_payment' : IDL.Func([AddPaymentRequest], [AddPaymentResponse], []),
    'add_stable_asset' : IDL.Func([IDL.Vec(IDL.Nat8)], [], []),
    'get_stats' : IDL.Func([], [Stats], ['query']),
    'refund_payment' : IDL.Func(
        [RefundPaymentRequest],
        [RefundPaymentResponse],
        [],
      ),
    'sync_icp_payment' : IDL.Func(
        [SyncICPPaymentRequest],
        [VerifyPaymentResponse],
        [],
      ),
    'verify_payment' : IDL.Func(
        [VerifyPaymentRequest],
        [VerifyPaymentResponse],
        ['query'],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
