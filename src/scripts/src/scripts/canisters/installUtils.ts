import { InitArgs, } from '~/declarations/registrar/registrar.did'
import { canister } from '../utils'
import { IDL } from '@dfinity/candid'
const init = ({ IDL }) => {
    const CanisterNames = IDL.Variant({
        'NamingMarketplace': IDL.Null,
        'RegistrarControlGateway': IDL.Null,
        'DICP': IDL.Null,
        'CyclesMinting': IDL.Null,
        'Registrar': IDL.Null,
        'MysteryBox': IDL.Null,
        'Registry': IDL.Null,
        'Ledger': IDL.Null,
        'Favorites': IDL.Null,
        'Resolver': IDL.Null,
    });
    const InitArgs = IDL.Record({
        'dev_named_canister_ids': IDL.Vec(IDL.Tuple(CanisterNames, IDL.Principal)),
    });
    return [IDL.Opt(InitArgs)];
};


export const reinstall_with_dev_ids = async (name: string) => {
    let initArgs: InitArgs = {
        dev_named_canister_ids: [],
    }
    const args = IDL.encode(init({ IDL }), [[initArgs]])
    await canister.reinstall_code(name, args)
}