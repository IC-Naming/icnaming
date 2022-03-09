export const idlFactory = ({ IDL }) => {
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
  const GetStatsActorResponse = IDL.Variant({
    'Ok' : Stats,
    'Err' : ErrorInfo,
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
    'get_stats' : IDL.Func([], [GetStatsActorResponse], ['query']),
    'import_quota' : IDL.Func([IDL.Vec(IDL.Nat8)], [ImportQuotaResponse], []),
    'load_state' : IDL.Func([StateExportData], [BooleanActorResponse], []),
  });
};
export const init = ({ IDL }) => { return []; };
