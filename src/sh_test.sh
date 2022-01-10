current_identity=$(dfx identity get-principal)
registry_id=$(dfx canister id registry)
registrar_id=$(dfx canister id registrar)
resolver_id=$(dfx canister id resolver)
favorites_id=$(dfx canister id favorites)
echo "calling registrar available"
dfx canister call registrar available '("wow.icp")'
echo "calling registrar available"
dfx canister call registrar available '("wow2.icp")'
echo "calling registrar available"
dfx canister call registrar available '("wow2*/.icp")'
echo "calling registrar available"
dfx canister call registrar available '("woww.icp")'

echo "calling registrar available"
dfx canister call registrar available '("woww.icp")'
echo "calling registrar register_for"
dfx canister call registrar register_for "(\"woww.icp\", principal \"$current_identity\", 10)"
echo "calling registrar register_with_quota"
dfx canister call registrar register_with_quota "(\"woww.icp\", variant { LenGte = 4})"

echo "calling registrar get_quota"
dfx canister call registrar get_quota "(principal \"$current_identity\", variant { LenGte = 4})"
echo "calling registrar add_quota"
dfx canister call registrar add_quota "(principal \"$current_identity\", variant { LenGte = 4}, 10)"
echo "calling registrar add_quota"
dfx canister call registrar add_quota "(principal \"$current_identity\", variant { LenGte = 4}, 10)"
echo "calling registrar get_quota"
dfx canister call registrar get_quota "(principal \"$current_identity\", variant { LenGte = 4})"
echo "calling registrar sub_quota"
dfx canister call registrar sub_quota "(principal \"$current_identity\", variant { LenGte = 4}, 10)"
echo "calling registrar get_quota"
dfx canister call registrar get_quota "(principal \"$current_identity\", variant { LenGte = 4})"
echo "calling registrar sub_quota"
dfx canister call registrar sub_quota "(principal \"$current_identity\", variant { LenGte = 4}, 11)"

echo "calling registrar available"
dfx canister call registrar available '("woww.icp")'
echo "calling registrar register_for"
dfx canister call registrar register_for "(\"woww1.icp\", principal \"$current_identity\", 10)"
echo "calling registrar register_with_quota"
dfx canister call registrar register_with_quota "(\"woww.icp\", variant { LenGte = 4})"
echo "calling registrar get_name_expires"
dfx canister call registrar get_name_expires '("woww.icp")'
echo "calling registrar get_details"
dfx canister call registrar get_details '("woww.icp")'
echo "calling registrar get_owner"
dfx canister call registrar get_owner '("woww.icp")'

echo "calling registry get_resolver"
dfx canister call registry get_resolver '("woww.icp")'
echo "calling registry get_owner"
dfx canister call registry get_owner '("woww.icp")'
echo "calling registry get_ttl"
dfx canister call registry get_ttl '("woww.icp")'
echo "calling registry get_details"
dfx canister call registry get_details '("woww.icp")'

echo "calling resolver set_record_value"
dfx canister call resolver set_record_value '("woww.icp", vec { record {"com.github"; "nice"}; record {"email"; "nice"}; record {"canister.icp"; "lzj7c-ayaaa-aaaad-qalna-cai"} })'
dfx canister call resolver get_record_value '("woww.icp")'


curl -L "http://127.0.0.1:8000/?canisterId=$registry_id&name=woww.icp&key=com.github"
curl -L "http://127.0.0.1:8000/?canisterId=$resolver_id&name=woww.icp&key=com.github"

echo "calling favorites get_favorites"
dfx canister call favorites get_favorites '()'
echo "calling favorites add_favorite"
dfx canister call favorites add_favorite '("nice.icp")'
echo "calling favorites add_favorite"
dfx canister call favorites add_favorite '("")'
echo "calling favorites add_favorite"
dfx canister call favorites add_favorite '("nice..icp")'
echo "calling favorites add_favorite"
dfx canister call favorites add_favorite '("nice.你.icp")'
echo "calling favorites get_favorites"
dfx canister call favorites get_favorites '()'
echo "calling favorites add_favorite"
dfx canister call favorites add_favorite '("nice2.icp")'
echo "calling favorites get_favorites"
dfx canister call favorites get_favorites '()'
echo "calling favorites remove_favorite"
dfx canister call favorites remove_favorite '("nice.icp")'
echo "calling favorites remove_favorite"
dfx canister call favorites remove_favorite '("nice2.icp")'
echo "calling favorites get_favorites"
dfx canister call favorites get_favorites '()'


dfx canister call registrar get_all_details '(record { offset=0 ; limit=100 })'