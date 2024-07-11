"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sha224 = void 0;
const sha256_1 = require("@noble/hashes/sha256");
/**
 * Returns the SHA224 hash of the buffer.
 * @param data Arraybuffer to encode
 */
function sha224(data) {
    return sha256_1.sha224.create().update(new Uint8Array(data)).digest();
}
exports.sha224 = sha224;
//# sourceMappingURL=sha224.js.map