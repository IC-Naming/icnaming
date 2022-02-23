export const idlFactory = ({ IDL }) => {
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
    'set_approval' : IDL.Func(
        [IDL.Text, IDL.Principal, IDL.Bool],
        [BooleanActorResponse],
        [],
      ),
    'set_record' : IDL.Func(
        [IDL.Text, IDL.Nat64, IDL.Principal],
        [BooleanActorResponse],
        [],
      ),
    'set_subdomain_owner' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Principal, IDL.Nat64, IDL.Principal],
        [GetDetailsResponse],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
