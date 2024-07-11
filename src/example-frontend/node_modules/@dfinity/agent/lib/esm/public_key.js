var __classPrivateFieldSet = (this && this.__classPrivateFieldSet) || function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _Ed25519PublicKey_rawKey, _Ed25519PublicKey_derKey;
import { ED25519_OID, unwrapDER, wrapDER } from './der';
export class Ed25519PublicKey {
    // `fromRaw` and `fromDer` should be used for instantiation, not this constructor.
    constructor(key) {
        _Ed25519PublicKey_rawKey.set(this, void 0);
        _Ed25519PublicKey_derKey.set(this, void 0);
        if (key.byteLength !== Ed25519PublicKey.RAW_KEY_LENGTH) {
            throw new Error('An Ed25519 public key must be exactly 32bytes long');
        }
        __classPrivateFieldSet(this, _Ed25519PublicKey_rawKey, key, "f");
        __classPrivateFieldSet(this, _Ed25519PublicKey_derKey, Ed25519PublicKey.derEncode(key), "f");
    }
    static from(key) {
        return this.fromDer(key.toDer());
    }
    static fromRaw(rawKey) {
        return new Ed25519PublicKey(rawKey);
    }
    static fromDer(derKey) {
        return new Ed25519PublicKey(this.derDecode(derKey));
    }
    static derEncode(publicKey) {
        return wrapDER(publicKey, ED25519_OID).buffer;
    }
    static derDecode(key) {
        const unwrapped = unwrapDER(key, ED25519_OID);
        if (unwrapped.length !== this.RAW_KEY_LENGTH) {
            throw new Error('An Ed25519 public key must be exactly 32bytes long');
        }
        return unwrapped;
    }
    get rawKey() {
        return __classPrivateFieldGet(this, _Ed25519PublicKey_rawKey, "f");
    }
    get derKey() {
        return __classPrivateFieldGet(this, _Ed25519PublicKey_derKey, "f");
    }
    toDer() {
        return this.derKey;
    }
    toRaw() {
        return this.rawKey;
    }
}
_Ed25519PublicKey_rawKey = new WeakMap(), _Ed25519PublicKey_derKey = new WeakMap();
// The length of Ed25519 public keys is always 32 bytes.
Ed25519PublicKey.RAW_KEY_LENGTH = 32;
//# sourceMappingURL=public_key.js.map