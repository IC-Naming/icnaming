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
  const ImportQuotaResult = IDL.Variant({
    'Ok' : IDL.Null,
    'AlreadyExists' : IDL.Null,
    'InvalidRequest' : IDL.Null,
  });
  const ImportQuotaResponse = IDL.Variant({
    'Ok' : ImportQuotaResult,
    'Err' : ErrorInfo,
  });
  return IDL.Service({
    'assign_name' : IDL.Func(
        [IDL.Text, IDL.Principal],
        [AssignNameResponse],
        [],
      ),
    'export_state' : IDL.Func([], [StateExportResponse], []),
    'import_quota' : IDL.Func([IDL.Vec(IDL.Nat8)], [ImportQuotaResponse], []),
  });
};
export const init = ({ IDL }) => { return []; };
