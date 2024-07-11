"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.makeNonce = exports.SubmitRequestType = void 0;
const random_1 = require("../../utils/random");
// The types of values allowed in the `request_type` field for submit requests.
var SubmitRequestType;
(function (SubmitRequestType) {
    SubmitRequestType["Call"] = "call";
})(SubmitRequestType = exports.SubmitRequestType || (exports.SubmitRequestType = {}));
/**
 * Create a random Nonce, based on random values
 */
function makeNonce() {
    // Encode 128 bits.
    const buffer = new ArrayBuffer(16);
    const view = new DataView(buffer);
    const rand1 = (0, random_1.randomNumber)();
    const rand2 = (0, random_1.randomNumber)();
    const rand3 = (0, random_1.randomNumber)();
    const rand4 = (0, random_1.randomNumber)();
    view.setUint32(0, rand1);
    view.setUint32(4, rand2);
    view.setUint32(8, rand3);
    view.setUint32(12, rand4);
    return buffer;
}
exports.makeNonce = makeNonce;
//# sourceMappingURL=types.js.map