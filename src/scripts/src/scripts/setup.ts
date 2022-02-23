import {HttpAgent} from "@dfinity/agent";
import {identities} from "./utils/identity";
import {Given, setDefaultTimeout, Then, When} from "@cucumber/cucumber";

// This file may be used to polyfill features that aren't available in the test
// environment, i.e. JSDom.
//
// We sometimes need to do this because our target browsers are expected to have
// a feature that JSDom doesn't.
//
// Note that we can use webpack configuration to make some features available to
// Node.js in a similar way.

global.crypto = require('@trust/webcrypto');
global.TextEncoder = require('text-encoding').TextEncoder; // eslint-disable-line
global.TextDecoder = require('text-encoding').TextDecoder; // eslint-disable-line
global.fetch = require('node-fetch');
BigInt.prototype.toJSON = function () {
    return this.toString()
}
global.ic = {
    agent: new HttpAgent({
        host: "http://127.0.0.1:8000",
        identity: identities.main.identity,
    })
};


setDefaultTimeout(60 * 1000);
