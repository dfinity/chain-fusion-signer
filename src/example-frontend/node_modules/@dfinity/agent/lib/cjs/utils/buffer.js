"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.uint8ToBuf = exports.bufEquals = exports.compare = exports.fromHex = exports.toHex = exports.concat = void 0;
/**
 * Concatenate multiple array buffers.
 * @param buffers The buffers to concatenate.
 */
function concat(...buffers) {
    const result = new Uint8Array(buffers.reduce((acc, curr) => acc + curr.byteLength, 0));
    let index = 0;
    for (const b of buffers) {
        result.set(new Uint8Array(b), index);
        index += b.byteLength;
    }
    return result.buffer;
}
exports.concat = concat;
/**
 * Transforms a buffer to an hexadecimal string. This will use the buffer as an Uint8Array.
 * @param buffer The buffer to return the hexadecimal string of.
 */
function toHex(buffer) {
    return [...new Uint8Array(buffer)].map(x => x.toString(16).padStart(2, '0')).join('');
}
exports.toHex = toHex;
const hexRe = new RegExp(/^([0-9A-F]{2})*$/i);
/**
 * Transforms a hexadecimal string into an array buffer.
 * @param hex The hexadecimal string to use.
 */
function fromHex(hex) {
    if (!hexRe.test(hex)) {
        throw new Error('Invalid hexadecimal string.');
    }
    const buffer = [...hex]
        .reduce((acc, curr, i) => {
        acc[(i / 2) | 0] = (acc[(i / 2) | 0] || '') + curr;
        return acc;
    }, [])
        .map(x => Number.parseInt(x, 16));
    return new Uint8Array(buffer).buffer;
}
exports.fromHex = fromHex;
/**
 *
 * @param b1 array buffer 1
 * @param b2 array buffer 2
 * @returns number - negative if b1 < b2, positive if b1 > b2, 0 if b1 === b2
 */
function compare(b1, b2) {
    if (b1.byteLength !== b2.byteLength) {
        return b1.byteLength - b2.byteLength;
    }
    const u1 = new Uint8Array(b1);
    const u2 = new Uint8Array(b2);
    for (let i = 0; i < u1.length; i++) {
        if (u1[i] !== u2[i]) {
            return u1[i] - u2[i];
        }
    }
    return 0;
}
exports.compare = compare;
/**
 * Checks two array buffers for equality.
 * @param b1 array buffer 1
 * @param b2 array buffer 2
 * @returns boolean
 */
function bufEquals(b1, b2) {
    return compare(b1, b2) === 0;
}
exports.bufEquals = bufEquals;
/**
 * Returns a true ArrayBuffer from a Uint8Array, as Uint8Array.buffer is unsafe.
 * @param {Uint8Array} arr Uint8Array to convert
 * @returns ArrayBuffer
 */
function uint8ToBuf(arr) {
    return new DataView(arr.buffer, arr.byteOffset, arr.byteLength).buffer;
}
exports.uint8ToBuf = uint8ToBuf;
//# sourceMappingURL=buffer.js.map