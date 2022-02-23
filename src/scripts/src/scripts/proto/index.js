"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.loadProto = void 0;
var protobufjs_1 = require("protobufjs");
var path_1 = require("path");
var loadProto = function () {
    return (0, protobufjs_1.loadSync)([
        (0, path_1.resolve)(__dirname, "./ic_base_types.proto"),
        (0, path_1.resolve)(__dirname, "./ic_nns_common.proto"),
        (0, path_1.resolve)(__dirname, "./ic_ledger.proto"),
        (0, path_1.resolve)(__dirname, "./governance.proto"),
    ]);
};
exports.loadProto = loadProto;
