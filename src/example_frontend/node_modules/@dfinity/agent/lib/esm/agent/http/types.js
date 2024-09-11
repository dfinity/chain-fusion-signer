import { randomNumber } from '../../utils/random';
// The types of values allowed in the `request_type` field for submit requests.
export var SubmitRequestType;
(function (SubmitRequestType) {
    SubmitRequestType["Call"] = "call";
})(SubmitRequestType || (SubmitRequestType = {}));
/**
 * Create a random Nonce, based on random values
 */
export function makeNonce() {
    // Encode 128 bits.
    const buffer = new ArrayBuffer(16);
    const view = new DataView(buffer);
    const rand1 = randomNumber();
    const rand2 = randomNumber();
    const rand3 = randomNumber();
    const rand4 = randomNumber();
    view.setUint32(0, rand1);
    view.setUint32(4, rand2);
    view.setUint32(8, rand3);
    view.setUint32(12, rand4);
    return buffer;
}
//# sourceMappingURL=types.js.map