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
var _ExpirableMap_inner, _ExpirableMap_expirationTime, _a, _b;
/**
 * A map that expires entries after a given time.
 * Defaults to 10 minutes.
 */
export class ExpirableMap {
    /**
     * Create a new ExpirableMap.
     * @param {ExpirableMapOptions<any, any>} options - options for the map.
     * @param {Iterable<[any, any]>} options.source - an optional source of entries to initialize the map with.
     * @param {number} options.expirationTime - the time in milliseconds after which entries will expire.
     */
    constructor(options = {}) {
        // Internals
        _ExpirableMap_inner.set(this, void 0);
        _ExpirableMap_expirationTime.set(this, void 0);
        this[_a] = this.entries.bind(this);
        this[_b] = 'ExpirableMap';
        const { source = [], expirationTime = 10 * 60 * 1000 } = options;
        const currentTime = Date.now();
        __classPrivateFieldSet(this, _ExpirableMap_inner, new Map([...source].map(([key, value]) => [key, { value, timestamp: currentTime }])), "f");
        __classPrivateFieldSet(this, _ExpirableMap_expirationTime, expirationTime, "f");
    }
    /**
     * Prune removes all expired entries.
     */
    prune() {
        const currentTime = Date.now();
        for (const [key, entry] of __classPrivateFieldGet(this, _ExpirableMap_inner, "f").entries()) {
            if (currentTime - entry.timestamp > __classPrivateFieldGet(this, _ExpirableMap_expirationTime, "f")) {
                __classPrivateFieldGet(this, _ExpirableMap_inner, "f").delete(key);
            }
        }
        return this;
    }
    // Implementing the Map interface
    /**
     * Set the value for the given key. Prunes expired entries.
     * @param key for the entry
     * @param value of the entry
     * @returns this
     */
    set(key, value) {
        this.prune();
        const entry = {
            value,
            timestamp: Date.now(),
        };
        __classPrivateFieldGet(this, _ExpirableMap_inner, "f").set(key, entry);
        return this;
    }
    /**
     * Get the value associated with the key, if it exists and has not expired.
     * @param key K
     * @returns the value associated with the key, or undefined if the key is not present or has expired.
     */
    get(key) {
        const entry = __classPrivateFieldGet(this, _ExpirableMap_inner, "f").get(key);
        if (entry === undefined) {
            return undefined;
        }
        if (Date.now() - entry.timestamp > __classPrivateFieldGet(this, _ExpirableMap_expirationTime, "f")) {
            __classPrivateFieldGet(this, _ExpirableMap_inner, "f").delete(key);
            return undefined;
        }
        return entry.value;
    }
    /**
     * Clear all entries.
     */
    clear() {
        __classPrivateFieldGet(this, _ExpirableMap_inner, "f").clear();
    }
    /**
     * Entries returns the entries of the map, without the expiration time.
     * @returns an iterator over the entries of the map.
     */
    entries() {
        const iterator = __classPrivateFieldGet(this, _ExpirableMap_inner, "f").entries();
        const generator = function* () {
            for (const [key, value] of iterator) {
                yield [key, value.value];
            }
        };
        return generator();
    }
    /**
     * Values returns the values of the map, without the expiration time.
     * @returns an iterator over the values of the map.
     */
    values() {
        const iterator = __classPrivateFieldGet(this, _ExpirableMap_inner, "f").values();
        const generator = function* () {
            for (const value of iterator) {
                yield value.value;
            }
        };
        return generator();
    }
    /**
     * Keys returns the keys of the map
     * @returns an iterator over the keys of the map.
     */
    keys() {
        return __classPrivateFieldGet(this, _ExpirableMap_inner, "f").keys();
    }
    /**
     * forEach calls the callbackfn on each entry of the map.
     * @param callbackfn to call on each entry
     * @param thisArg to use as this when calling the callbackfn
     */
    forEach(callbackfn, thisArg) {
        for (const [key, value] of __classPrivateFieldGet(this, _ExpirableMap_inner, "f").entries()) {
            callbackfn.call(thisArg, value.value, key, this);
        }
    }
    /**
     * has returns true if the key exists and has not expired.
     * @param key K
     * @returns true if the key exists and has not expired.
     */
    has(key) {
        return __classPrivateFieldGet(this, _ExpirableMap_inner, "f").has(key);
    }
    /**
     * delete the entry for the given key.
     * @param key K
     * @returns true if the key existed and has been deleted.
     */
    delete(key) {
        return __classPrivateFieldGet(this, _ExpirableMap_inner, "f").delete(key);
    }
    /**
     * get size of the map.
     * @returns the size of the map.
     */
    get size() {
        return __classPrivateFieldGet(this, _ExpirableMap_inner, "f").size;
    }
}
_ExpirableMap_inner = new WeakMap(), _ExpirableMap_expirationTime = new WeakMap(), _a = Symbol.iterator, _b = Symbol.toStringTag;
//# sourceMappingURL=expirableMap.js.map