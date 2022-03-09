export const idlFactory = ({ IDL }) => {
  const ErrorInfo = IDL.Record({ 'code' : IDL.Nat32, 'message' : IDL.Text });
  const BooleanActorResponse = IDL.Variant({
    'Ok' : IDL.Bool,
    'Err' : ErrorInfo,
  });
  const StateExportData = IDL.Record({ 'state_data' : IDL.Vec(IDL.Nat8) });
  const StateExportResponse = IDL.Variant({
    'Ok' : StateExportData,
    'Err' : ErrorInfo,
  });
  const GetRecordValueResponse = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'Err' : ErrorInfo,
  });
  const Stats = IDL.Record({
    'cycles_balance' : IDL.Nat64,
    'resolver_count' : IDL.Nat64,
  });
  const GetStatsResponse = IDL.Variant({ 'Ok' : Stats, 'Err' : ErrorInfo });
  return IDL.Service({
    'ensure_resolver_created' : IDL.Func(
        [IDL.Text],
        [BooleanActorResponse],
        [],
      ),
    'export_state' : IDL.Func([], [StateExportResponse], []),
    'get_record_value' : IDL.Func(
        [IDL.Text],
        [GetRecordValueResponse],
        ['query'],
      ),
    'get_stats' : IDL.Func([], [GetStatsResponse], ['query']),
    'load_state' : IDL.Func([StateExportData], [BooleanActorResponse], []),
    'set_record_value' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))],
        [BooleanActorResponse],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
