export const idlFactory = ({ IDL }) => {
  const CanisterNames = IDL.Variant({
    'NamingMarketplace' : IDL.Null,
    'RegistrarControlGateway' : IDL.Null,
    'DICP' : IDL.Null,
    'CyclesMinting' : IDL.Null,
    'Registrar' : IDL.Null,
    'MysteryBox' : IDL.Null,
    'Registry' : IDL.Null,
    'Ledger' : IDL.Null,
    'Favorites' : IDL.Null,
    'Resolver' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'dev_named_canister_ids' : IDL.Vec(IDL.Tuple(CanisterNames, IDL.Principal)),
  });
  const QuotaType = IDL.Variant({ 'LenEq' : IDL.Nat8, 'LenGte' : IDL.Nat8 });
  const ErrorInfo = IDL.Record({ 'code' : IDL.Nat32, 'message' : IDL.Text });
  const BooleanActorResponse = IDL.Variant({
    'Ok' : IDL.Bool,
    'Err' : ErrorInfo,
  });
  const TransferQuotaDetails = IDL.Record({
    'to' : IDL.Principal,
    'diff' : IDL.Nat32,
    'quota_type' : QuotaType,
  });
  const BatchTransferRequest = IDL.Record({
    'items' : IDL.Vec(TransferQuotaDetails),
  });
  const StateExportData = IDL.Record({ 'state_data' : IDL.Vec(IDL.Nat8) });
  const StateExportResponse = IDL.Variant({
    'Ok' : StateExportData,
    'Err' : ErrorInfo,
  });
  const GetPageInput = IDL.Record({
    'offset' : IDL.Nat64,
    'limit' : IDL.Nat64,
  });
  const RegistrationDetails = IDL.Record({
    'owner' : IDL.Principal,
    'name' : IDL.Text,
    'created_at' : IDL.Nat64,
    'expired_at' : IDL.Nat64,
  });
  const GetAllDetailsActorResponse = IDL.Variant({
    'Ok' : IDL.Vec(RegistrationDetails),
    'Err' : ErrorInfo,
  });
  const GetDetailsActorResponse = IDL.Variant({
    'Ok' : RegistrationDetails,
    'Err' : ErrorInfo,
  });
  const GetNameExpiresActorResponse = IDL.Variant({
    'Ok' : IDL.Nat64,
    'Err' : ErrorInfo,
  });
  const RegistrationDto = IDL.Record({
    'name' : IDL.Text,
    'created_at' : IDL.Nat64,
    'expired_at' : IDL.Nat64,
  });
  const GetPageOutput = IDL.Record({ 'items' : IDL.Vec(RegistrationDto) });
  const GetNamesActorResponse = IDL.Variant({
    'Ok' : GetPageOutput,
    'Err' : ErrorInfo,
  });
  const GetOwnerActorResponse = IDL.Variant({
    'Ok' : IDL.Principal,
    'Err' : ErrorInfo,
  });
  const NameOrderStatus = IDL.Variant({
    'New' : IDL.Null,
    'WaitingToRefund' : IDL.Null,
    'Done' : IDL.Null,
    'Canceled' : IDL.Null,
  });
  const GetNameOrderResponse = IDL.Record({
    'status' : NameOrderStatus,
    'name' : IDL.Text,
    'created_at' : IDL.Nat64,
    'price_icp_in_e8s' : IDL.Nat,
    'created_user' : IDL.Principal,
    'years' : IDL.Nat32,
  });
  const GetPendingOrderActorResponse = IDL.Variant({
    'Ok' : IDL.Opt(GetNameOrderResponse),
    'Err' : ErrorInfo,
  });
  const PriceTableItem = IDL.Record({
    'len' : IDL.Nat8,
    'price_in_icp_e8s' : IDL.Nat64,
    'price_in_xdr_permyriad' : IDL.Nat64,
  });
  const PriceTable = IDL.Record({
    'icp_xdr_conversion_rate' : IDL.Nat64,
    'items' : IDL.Vec(PriceTableItem),
  });
  const GetPriceTableResponse = IDL.Variant({
    'Ok' : PriceTable,
    'Err' : ErrorInfo,
  });
  const GetPublicResolverActorResponse = IDL.Variant({
    'Ok' : IDL.Text,
    'Err' : ErrorInfo,
  });
  const GetQuotaActorResponse = IDL.Variant({
    'Ok' : IDL.Nat32,
    'Err' : ErrorInfo,
  });
  const Stats = IDL.Record({
    'user_count' : IDL.Nat64,
    'new_registered_name_count' : IDL.Nat64,
    'cycles_balance' : IDL.Nat64,
    'last_xdr_permyriad_per_icp' : IDL.Nat64,
    'name_order_cancelled_count' : IDL.Nat64,
    'user_quota_count' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64)),
    'name_order_placed_count' : IDL.Nat64,
    'name_order_paid_count' : IDL.Nat64,
    'user_name_order_count_by_status' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64)),
    'last_timestamp_seconds_xdr_permyriad_per_icp' : IDL.Nat64,
    'name_lock_count' : IDL.Nat64,
    'payment_version' : IDL.Nat64,
    'user_quota_order_count' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64)),
    'registration_count' : IDL.Nat64,
  });
  const GetStatsResponse = IDL.Variant({ 'Ok' : Stats, 'Err' : ErrorInfo });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const Token = IDL.Record({
    'key' : IDL.Text,
    'sha256' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'index' : IDL.Nat,
    'content_encoding' : IDL.Text,
  });
  const CallbackStrategy = IDL.Record({
    'token' : Token,
    'callback' : IDL.Func([], [], []),
  });
  const StreamingStrategy = IDL.Variant({ 'Callback' : CallbackStrategy });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'streaming_strategy' : IDL.Opt(StreamingStrategy),
    'status_code' : IDL.Nat16,
  });
  const ImportQuotaItem = IDL.Record({
    'owner' : IDL.Principal,
    'diff' : IDL.Nat32,
    'quota_type' : IDL.Text,
  });
  const ImportQuotaRequest = IDL.Record({
    'hash' : IDL.Vec(IDL.Nat8),
    'items' : IDL.Vec(ImportQuotaItem),
  });
  const ImportQuotaStatus = IDL.Variant({
    'Ok' : IDL.Null,
    'AlreadyExists' : IDL.Null,
  });
  const ImportQuotaResponse = IDL.Variant({
    'Ok' : ImportQuotaStatus,
    'Err' : ErrorInfo,
  });
  const ImportNameRegistrationItem = IDL.Record({
    'owner' : IDL.Principal,
    'name' : IDL.Text,
    'years' : IDL.Nat32,
  });
  const ImportNameRegistrationRequest = IDL.Record({
    'items' : IDL.Vec(ImportNameRegistrationItem),
  });
  const RenewNameRequest = IDL.Record({
    'name' : IDL.Text,
    'approve_amount' : IDL.Nat64,
    'years' : IDL.Nat32,
  });
  const SubmitOrderRequest = IDL.Record({
    'name' : IDL.Text,
    'years' : IDL.Nat32,
  });
  const SubmitOrderResponse = IDL.Record({ 'order' : GetNameOrderResponse });
  const SubmitOrderActorResponse = IDL.Variant({
    'Ok' : SubmitOrderResponse,
    'Err' : ErrorInfo,
  });
  const TransferFromQuotaRequest = IDL.Record({
    'to' : IDL.Principal,
    'diff' : IDL.Nat32,
    'from' : IDL.Principal,
    'quota_type' : QuotaType,
  });
  return IDL.Service({
    'add_quota' : IDL.Func(
        [IDL.Principal, QuotaType, IDL.Nat32],
        [BooleanActorResponse],
        [],
      ),
    'approve' : IDL.Func([IDL.Text, IDL.Principal], [BooleanActorResponse], []),
    'available' : IDL.Func([IDL.Text], [BooleanActorResponse], ['query']),
    'batch_transfer_quota' : IDL.Func(
        [BatchTransferRequest],
        [BooleanActorResponse],
        [],
      ),
    'cancel_order' : IDL.Func([], [BooleanActorResponse], []),
    'export_state' : IDL.Func([], [StateExportResponse], []),
    'get_all_details' : IDL.Func(
        [GetPageInput],
        [GetAllDetailsActorResponse],
        ['query'],
      ),
    'get_details' : IDL.Func([IDL.Text], [GetDetailsActorResponse], ['query']),
    'get_last_registrations' : IDL.Func(
        [],
        [GetAllDetailsActorResponse],
        ['query'],
      ),
    'get_name_expires' : IDL.Func(
        [IDL.Text],
        [GetNameExpiresActorResponse],
        ['query'],
      ),
    'get_names' : IDL.Func(
        [IDL.Principal, GetPageInput],
        [GetNamesActorResponse],
        ['query'],
      ),
    'get_owner' : IDL.Func([IDL.Text], [GetOwnerActorResponse], ['query']),
    'get_pending_order' : IDL.Func(
        [],
        [GetPendingOrderActorResponse],
        ['query'],
      ),
    'get_price_table' : IDL.Func([], [GetPriceTableResponse], []),
    'get_public_resolver' : IDL.Func(
        [],
        [GetPublicResolverActorResponse],
        ['query'],
      ),
    'get_quota' : IDL.Func(
        [IDL.Principal, QuotaType],
        [GetQuotaActorResponse],
        ['query'],
      ),
    'get_stats' : IDL.Func([], [GetStatsResponse], ['query']),
    'get_wasm_info' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))],
        ['query'],
      ),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'import_quota' : IDL.Func([ImportQuotaRequest], [ImportQuotaResponse], []),
    'import_registrations' : IDL.Func(
        [ImportNameRegistrationRequest],
        [BooleanActorResponse],
        [],
      ),
    'load_state' : IDL.Func([StateExportData], [BooleanActorResponse], []),
    'pay_my_order' : IDL.Func([], [BooleanActorResponse], []),
    'reclaim_name' : IDL.Func([IDL.Text], [BooleanActorResponse], []),
    'register_for' : IDL.Func(
        [IDL.Text, IDL.Principal, IDL.Nat64],
        [BooleanActorResponse],
        [],
      ),
    'register_from_gateway' : IDL.Func(
        [IDL.Text, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
    'register_with_quota' : IDL.Func(
        [IDL.Text, QuotaType],
        [BooleanActorResponse],
        [],
      ),
    'renew_name' : IDL.Func([RenewNameRequest], [BooleanActorResponse], []),
    'run_tasks' : IDL.Func([], [BooleanActorResponse], []),
    'sub_quota' : IDL.Func(
        [IDL.Principal, QuotaType, IDL.Nat32],
        [BooleanActorResponse],
        [],
      ),
    'submit_order' : IDL.Func(
        [SubmitOrderRequest],
        [SubmitOrderActorResponse],
        [],
      ),
    'transfer' : IDL.Func(
        [IDL.Text, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
    'transfer_by_admin' : IDL.Func(
        [IDL.Text, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
    'transfer_from' : IDL.Func([IDL.Text], [BooleanActorResponse], []),
    'transfer_from_quota' : IDL.Func(
        [TransferFromQuotaRequest],
        [BooleanActorResponse],
        [],
      ),
    'transfer_quota' : IDL.Func(
        [IDL.Principal, QuotaType, IDL.Nat32],
        [BooleanActorResponse],
        [],
      ),
    'unlock_names' : IDL.Func([IDL.Vec(IDL.Text)], [BooleanActorResponse], []),
  });
};
export const init = ({ IDL }) => {
  const CanisterNames = IDL.Variant({
    'NamingMarketplace' : IDL.Null,
    'RegistrarControlGateway' : IDL.Null,
    'DICP' : IDL.Null,
    'CyclesMinting' : IDL.Null,
    'Registrar' : IDL.Null,
    'MysteryBox' : IDL.Null,
    'Registry' : IDL.Null,
    'Ledger' : IDL.Null,
    'Favorites' : IDL.Null,
    'Resolver' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'dev_named_canister_ids' : IDL.Vec(IDL.Tuple(CanisterNames, IDL.Principal)),
  });
  return [IDL.Opt(InitArgs)];
};
