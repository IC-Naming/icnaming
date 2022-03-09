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
  const GetFavoritesResponse = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Text),
    'Err' : ErrorInfo,
  });
  const Stats = IDL.Record({
    'user_count' : IDL.Nat64,
    'cycles_balance' : IDL.Nat64,
    'favorite_count' : IDL.Nat64,
  });
  const GetStatsResponse = IDL.Variant({ 'Ok' : Stats, 'Err' : ErrorInfo });
  return IDL.Service({
    'add_favorite' : IDL.Func([IDL.Text], [BooleanActorResponse], []),
    'export_state' : IDL.Func([], [StateExportResponse], []),
    'get_favorites' : IDL.Func([], [GetFavoritesResponse], ['query']),
    'get_stats' : IDL.Func([], [GetStatsResponse], ['query']),
    'load_state' : IDL.Func([StateExportData], [BooleanActorResponse], []),
    'remove_favorite' : IDL.Func([IDL.Text], [BooleanActorResponse], []),
  });
};
export const init = ({ IDL }) => { return []; };
