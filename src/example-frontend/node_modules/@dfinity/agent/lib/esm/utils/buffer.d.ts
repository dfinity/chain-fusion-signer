/**
 * Concatenate multiple array buffers.
 * @param buffers The buffers to concatenate.
 */
export declare function concat(...buffers: ArrayBuffer[]): ArrayBuffer;
/**
 * Transforms a buffer to an hexadecimal string. This will use the buffer as an Uint8Array.
 * @param buffer The buffer to return the hexadecimal string of.
 */
export declare function toHex(buffer: ArrayBuffer): string;
/**
 * Transforms a hexadecimal string into an array buffer.
 * @param hex The hexadecimal string to use.
 */
export declare function fromHex(hex: string): ArrayBuffer;
/**
 *
 * @param b1 array buffer 1
 * @param b2 array buffer 2
 * @returns number - negative if b1 < b2, positive if b1 > b2, 0 if b1 === b2
 */
export declare function compare(b1: ArrayBuffer, b2: ArrayBuffer): number;
/**
 * Checks two array buffers for equality.
 * @param b1 array buffer 1
 * @param b2 array buffer 2
 * @returns boolean
 */
export declare function bufEquals(b1: ArrayBuffer, b2: ArrayBuffer): boolean;
/**
 * Returns a true ArrayBuffer from a Uint8Array, as Uint8Array.buffer is unsafe.
 * @param {Uint8Array} arr Uint8Array to convert
 * @returns ArrayBuffer
 */
export declare function uint8ToBuf(arr: Uint8Array): ArrayBuffer;
