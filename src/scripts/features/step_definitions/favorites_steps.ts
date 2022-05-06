import "~/setup";
import {Then, When} from "@cucumber/cucumber";
import {identities} from "~/utils/identity";
import {createFavorites} from "~/declarations/favorites";
import {Result} from "~/utils/Result";
import {expect} from "chai";

When(/^User "([^"]*)" add some favorites$/,
    async function (user: string, data) {
        const identityInfo = identities.get_identity_info(user);
        const favorites = createFavorites(identityInfo);
        const rows = data.rows();
        for (const row of rows) {
            const item = row[0];
            await new Result(favorites.add_favorite(item)).unwrap();
        }
    });
Then(/^User "([^"]*)" should see the favorites$/,
    async function (user: string, data) {
        const identityInfo = identities.get_identity_info(user);
        const favorites = createFavorites(identityInfo);
        const result = await new Result(favorites.get_favorites()).unwrap();

        const rows = data.rows();
        expect(result.length).to.equal(rows.length);
        for (const row of rows) {
            const item = row[0];
            expect(result).to.include(item);
        }
    });
When(/^User "([^"]*)" delete a favorite "([^"]*)"$/,
    async function (user: string, favorite: string) {
        const identityInfo = identities.get_identity_info(user);
        const favorites = createFavorites(identityInfo);
        await new Result(favorites.remove_favorite(favorite)).unwrap();
    });
