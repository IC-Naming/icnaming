import type { Principal } from '@dfinity/principal';
export type BooleanActorResponse = { 'Ok' : boolean } |
  { 'Err' : ErrorInfo };
export interface ErrorInfo { 'code' : number, 'message' : string }
export type GetFavoritesResponse = { 'Ok' : Array<string> } |
  { 'Err' : ErrorInfo };
export type GetStatsResponse = { 'Ok' : Stats } |
  { 'Err' : ErrorInfo };
export interface StateExportData { 'state_data' : Array<number> }
export type StateExportResponse = { 'Ok' : StateExportData } |
  { 'Err' : ErrorInfo };
export interface Stats {
  'user_count' : bigint,
  'cycles_balance' : bigint,
  'favorite_count' : bigint,
}
export interface _SERVICE {
  'add_favorite' : (arg_0: string) => Promise<BooleanActorResponse>,
  'export_state' : () => Promise<StateExportResponse>,
  'get_favorites' : () => Promise<GetFavoritesResponse>,
  'get_stats' : () => Promise<GetStatsResponse>,
  'load_state' : (arg_0: StateExportData) => Promise<BooleanActorResponse>,
  'remove_favorite' : (arg_0: string) => Promise<BooleanActorResponse>,
}
