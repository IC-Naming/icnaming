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
    'cycles_balance' : IDL.Nat64,
    'latest_transaction_block_height' : BlockHeight,
    'seconds_since_last_ledger_sync' : IDL.Nat64,
    'payments_version' : IDL.Nat64,
    'count_of_payments_by_status' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64)),
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
  const SyncICPPaymentResponse = IDL.Record({
    'verify_payment_response' : IDL.Opt(VerifyPaymentResponse),
    'payment_id' : IDL.Opt(PaymentId),
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
        [SyncICPPaymentResponse],
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
