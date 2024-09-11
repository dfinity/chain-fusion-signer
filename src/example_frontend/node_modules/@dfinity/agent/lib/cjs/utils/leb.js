"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decodeTime = exports.decodeLeb128 = void 0;
const candid_1 = require("@dfinity/candid");
const decodeLeb128 = (buf) => {
    return (0, candid_1.lebDecode)(new candid_1.PipeArrayBuffer(buf));
};
exports.decodeLeb128 = decodeLeb128;
// time is a LEB128-encoded Nat
const decodeTime = (buf) => {
    const decoded = (0, exports.decodeLeb128)(buf);
    // nanoseconds to milliseconds
    return new Date(Number(decoded) / 1000000);
};
exports.decodeTime = decodeTime;
//# sourceMappingURL=leb.js.map