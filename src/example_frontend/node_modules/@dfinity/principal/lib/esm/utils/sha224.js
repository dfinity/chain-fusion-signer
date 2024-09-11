import { sha224 as jsSha224 } from '@noble/hashes/sha256';
/**
 * Returns the SHA224 hash of the buffer.
 * @param data Arraybuffer to encode
 */
export function sha224(data) {
    return jsSha224.create().update(new Uint8Array(data)).digest();
}
//# sourceMappingURL=sha224.js.map