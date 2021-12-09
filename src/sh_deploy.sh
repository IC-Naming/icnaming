dfx canister create --all
dfx build
echo yes | dfx canister install -m reinstall registry
echo yes | dfx canister install -m reinstall registrar
echo yes | dfx canister install -m reinstall resolver
echo yes | dfx canister install -m reinstall favorites
