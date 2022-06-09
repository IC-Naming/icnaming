import type { Principal } from '@dfinity/principal';
export type CyclesResponse = { 'Refunded' : [string, [] | [bigint]] } |
  { 'CanisterCreated' : Principal } |
  { 'ToppedUp' : null };
export interface ICPTs { 'e8s' : bigint }
export interface IcpXdrConversionRate {
  'xdr_permyriad_per_icp' : bigint,
  'timestamp_seconds' : bigint,
}
export interface IcpXdrConversionRateCertifiedResponse {
  'certificate' : Array<number>,
  'data' : IcpXdrConversionRate,
  'hash_tree' : Array<number>,
}
export type Result = { 'Ok' : CyclesResponse } |
  { 'Err' : string };
export interface SetAuthorizedSubnetworkListArgs {
  'who' : [] | [Principal],
  'subnets' : Array<Principal>,
}
export interface TransactionNotification {
  'to' : Principal,
  'to_subaccount' : [] | [Array<number>],
  'from' : Principal,
  'memo' : bigint,
  'from_subaccount' : [] | [Array<number>],
  'amount' : ICPTs,
  'block_height' : bigint,
}
export interface _SERVICE {
  'get_average_icp_xdr_conversion_rate' : () => Promise<
      IcpXdrConversionRateCertifiedResponse
    >,
  'get_icp_xdr_conversion_rate' : () => Promise<
      IcpXdrConversionRateCertifiedResponse
    >,
  'set_authorized_subnetwork_list' : (
      arg_0: SetAuthorizedSubnetworkListArgs,
    ) => Promise<undefined>,
  'transaction_notification' : (arg_0: TransactionNotification) => Promise<
      Result
    >,
}
