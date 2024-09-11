"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.randomNumber = void 0;
/**
 * Generates a random unsigned 32-bit integer between 0 and 0xffffffff
 * @returns {number} a random number
 */
const randomNumber = () => {
    // determine whether browser crypto is available
    if (typeof window !== 'undefined' && !!window.crypto && !!window.crypto.getRandomValues) {
        const array = new Uint32Array(1);
        window.crypto.getRandomValues(array);
        return array[0];
    }
    // A second check for webcrypto, in case it is loaded under global instead of window
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
        const array = new Uint32Array(1);
        crypto.getRandomValues(array);
        return array[0];
    }
    // determine whether node crypto is available
    if (typeof crypto !== 'undefined' && crypto.randomInt) {
        return crypto.randomInt(0, 0xffffffff);
    }
    // fall back to Math.random
    return Math.floor(Math.random() * 0xffffffff);
};
exports.randomNumber = randomNumber;
//# sourceMappingURL=random.js.map