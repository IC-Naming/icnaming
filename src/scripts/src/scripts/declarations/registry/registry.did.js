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
  const StateExportData = IDL.Record({ 'state_data' : IDL.Vec(IDL.Nat8) });
  const ErrorInfo = IDL.Record({ 'code' : IDL.Nat32, 'message' : IDL.Text });
  const StateExportResponse = IDL.Variant({
    'Ok' : StateExportData,
    'Err' : ErrorInfo,
  });
  const GetPageInput = IDL.Record({
    'offset' : IDL.Nat64,
    'limit' : IDL.Nat64,
  });
  const GetPageOutput = IDL.Record({ 'items' : IDL.Vec(IDL.Text) });
  const GetControlledNamesResponse = IDL.Variant({
    'Ok' : GetPageOutput,
    'Err' : ErrorInfo,
  });
  const RegistryDto = IDL.Record({
    'ttl' : IDL.Nat64,
    'resolver' : IDL.Principal,
    'owner' : IDL.Principal,
    'name' : IDL.Text,
  });
  const GetDetailsResponse = IDL.Variant({
    'Ok' : RegistryDto,
    'Err' : ErrorInfo,
  });
  const GetOwnerResponse = IDL.Variant({
    'Ok' : IDL.Principal,
    'Err' : ErrorInfo,
  });
  const Stats = IDL.Record({
    'cycles_balance' : IDL.Nat64,
    'registry_count' : IDL.Nat64,
  });
  const GetStatsResponse = IDL.Variant({ 'Ok' : Stats, 'Err' : ErrorInfo });
  const GetTtlResponse = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : ErrorInfo });
  const RegistryUsers = IDL.Record({
    'owner' : IDL.Principal,
    'operators' : IDL.Vec(IDL.Principal),
  });
  const GetUsersResponse = IDL.Variant({
    'Ok' : RegistryUsers,
    'Err' : ErrorInfo,
  });
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
  const BooleanActorResponse = IDL.Variant({
    'Ok' : IDL.Bool,
    'Err' : ErrorInfo,
  });
  return IDL.Service({
    'export_state' : IDL.Func([], [StateExportResponse], []),
    'get_controlled_names' : IDL.Func(
        [IDL.Principal, GetPageInput],
        [GetControlledNamesResponse],
        ['query'],
      ),
    'get_details' : IDL.Func([IDL.Text], [GetDetailsResponse], ['query']),
    'get_owner' : IDL.Func([IDL.Text], [GetOwnerResponse], ['query']),
    'get_resolver' : IDL.Func([IDL.Text], [GetOwnerResponse], ['query']),
    'get_stats' : IDL.Func([], [GetStatsResponse], ['query']),
    'get_ttl' : IDL.Func([IDL.Text], [GetTtlResponse], ['query']),
    'get_users' : IDL.Func([IDL.Text], [GetUsersResponse], ['query']),
    'get_wasm_info' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))],
        ['query'],
      ),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'load_state' : IDL.Func([StateExportData], [BooleanActorResponse], []),
    'reclaim_name' : IDL.Func(
        [IDL.Text, IDL.Principal, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
    'set_approval' : IDL.Func(
        [IDL.Text, IDL.Principal, IDL.Bool],
        [BooleanActorResponse],
        [],
      ),
    'set_owner' : IDL.Func(
        [IDL.Text, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
    'set_record' : IDL.Func(
        [IDL.Text, IDL.Nat64, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
    'set_resolver' : IDL.Func(
        [IDL.Text, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
    'set_subdomain_owner' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Principal, IDL.Nat64, IDL.Principal],
        [GetDetailsResponse],
        [],
      ),
    'transfer' : IDL.Func(
        [IDL.Text, IDL.Principal, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
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
