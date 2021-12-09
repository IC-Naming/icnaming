did=$(dfx canister call registry __get_candid_interface_tmp_hack)
did=${did#*"\""}
did=${did%"\""*}
echo "$did" > canisters/registry/src/registry.did 
echo "$did"
echo "==================registry"


did=$(dfx canister call registrar __get_candid_interface_tmp_hack)
did=${did#*"\""}
did=${did%"\""*}
echo "$did" > canisters/registrar/src/registrar.did 
echo "$did"
echo "==================registrar"


did=$(dfx canister call resolver __get_candid_interface_tmp_hack)
did=${did#*"\""}
did=${did%"\""*}
echo "$did" > canisters/resolver/src/resolver.did 
echo "$did"
echo "==================resolver"

did=$(dfx canister call favorites __get_candid_interface_tmp_hack)
did=${did#*"\""}
did=${did%"\""*}
echo "$did" > canisters/favorites/src/favorites.did
echo "$did"
echo "==================favorites"


echo "done"