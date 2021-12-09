current_identity=$(dfx identity get-principal)
registry_id=$(dfx canister id registry)
registrar_id=$(dfx canister id registrar)
resolver_id=$(dfx canister id resolver)
favorites_id=$(dfx canister id favorites)
dfx canister call registrar set_owner "(principal \"$current_identity\")"
dfx canister call registrar set_named "(\"ens_activity_client\", principal \"$current_identity\")"
dfx canister call registrar set_named "(\"registry\", principal \"$registry_id\")"
dfx canister call registrar set_named "(\"resolver\", principal \"$resolver_id\")"

dfx canister call registry set_owner "(principal \"$current_identity\")"
dfx canister call registry set_named "(\"registrar\", principal \"$registrar_id\")"
dfx canister call registry set_named "(\"resolver\", principal \"$resolver_id\")"
dfx canister call registry set_top_name

dfx canister call resolver set_owner "(principal \"$current_identity\")"
dfx canister call resolver set_named "(\"registry\", principal \"$registry_id\")"