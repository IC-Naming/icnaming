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
  const AssignNameResult = IDL.Variant({
    'Ok' : IDL.Null,
    'AlreadyAssigned' : IDL.Null,
    'FailFromRegistrar' : IDL.Null,
  });
  const ErrorInfo = IDL.Record({ 'code' : IDL.Nat32, 'message' : IDL.Text });
  const AssignNameResponse = IDL.Variant({
    'Ok' : AssignNameResult,
    'Err' : ErrorInfo,
  });
  const StateExportData = IDL.Record({ 'state_data' : IDL.Vec(IDL.Nat8) });
  const StateExportResponse = IDL.Variant({
    'Ok' : StateExportData,
    'Err' : ErrorInfo,
  });
  const Stats = IDL.Record({
    'name_assignments_count' : IDL.Nat64,
    'cycles_balance' : IDL.Nat64,
    'imported_file_hashes_count' : IDL.Nat64,
    'acceptable_file_hashes_count' : IDL.Nat64,
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
  const ImportQuotaResult = IDL.Variant({
    'Ok' : IDL.Null,
    'AlreadyExists' : IDL.Null,
    'InvalidRequest' : IDL.Null,
  });
  const ImportQuotaResponse = IDL.Variant({
    'Ok' : ImportQuotaResult,
    'Err' : ErrorInfo,
  });
  const BooleanActorResponse = IDL.Variant({
    'Ok' : IDL.Bool,
    'Err' : ErrorInfo,
  });
  return IDL.Service({
    'assign_name' : IDL.Func(
        [IDL.Text, IDL.Principal],
        [AssignNameResponse],
        [],
      ),
    'export_state' : IDL.Func([], [StateExportResponse], []),
    'get_stats' : IDL.Func([], [GetStatsResponse], ['query']),
    'get_wasm_info' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))],
        ['query'],
      ),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'import_quota' : IDL.Func([IDL.Vec(IDL.Nat8)], [ImportQuotaResponse], []),
    'load_state' : IDL.Func([StateExportData], [BooleanActorResponse], []),
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
