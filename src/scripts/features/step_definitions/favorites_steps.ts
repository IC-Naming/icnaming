import "~/setup";
import {Then, When} from "@cucumber/cucumber";
import {identities} from "~/utils/identity";
import {createFavorites} from "~/declarations/favorites";
import {Result} from "~/utils/Result";
import {expect} from "chai";

When(/^User "([^"]*)" add some favorites$/,
    async function (user: string, data) {
        let identityInfo = identities.get_identity_info(user);
        let favorites = createFavorites(identityInfo);
        let rows = data.rows();
        for (const row of rows) {
            let item = row[0];
            await new Result(favorites.add_favorite(item)).unwrap();
        }
    });
Then(/^User "([^"]*)" should see the favorites$/,
    async function (user: string, data) {
        let identityInfo = identities.get_identity_info(user);
        let favorites = createFavorites(identityInfo);
        let result = await new Result(favorites.get_favorites()).unwrap();

        let rows = data.rows();
        expect(result.length).to.equal(rows.length);
        for (const row of rows) {
            let item = row[0];
            expect(result).to.include(item);
        }
    });
When(/^User "([^"]*)" delete a favorite "([^"]*)"$/,
    async function (user: string, favorite: string) {
        let identityInfo = identities.get_identity_info(user);
        let favorites = createFavorites(identityInfo);
        await new Result(favorites.remove_favorite(favorite)).unwrap();
    });