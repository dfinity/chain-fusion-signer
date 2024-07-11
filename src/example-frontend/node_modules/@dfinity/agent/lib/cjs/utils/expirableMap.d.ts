export declare type ExpirableMapOptions<K, V> = {
    source?: Iterable<[K, V]>;
    expirationTime?: number;
};
/**
 * A map that expires entries after a given time.
 * Defaults to 10 minutes.
 */
export declare class ExpirableMap<K, V> implements Map<K, V> {
    #private;
    [Symbol.iterator]: () => IterableIterator<[K, V]>;
    [Symbol.toStringTag]: string;
    /**
     * Create a new ExpirableMap.
     * @param {ExpirableMapOptions<any, any>} options - options for the map.
     * @param {Iterable<[any, any]>} options.source - an optional source of entries to initialize the map with.
     * @param {number} options.expirationTime - the time in milliseconds after which entries will expire.
     */
    constructor(options?: ExpirableMapOptions<K, V>);
    /**
     * Prune removes all expired entries.
     */
    prune(): this;
    /**
     * Set the value for the given key. Prunes expired entries.
     * @param key for the entry
     * @param value of the entry
     * @returns this
     */
    set(key: K, value: V): this;
    /**
     * Get the value associated with the key, if it exists and has not expired.
     * @param key K
     * @returns the value associated with the key, or undefined if the key is not present or has expired.
     */
    get(key: K): V | undefined;
    /**
     * Clear all entries.
     */
    clear(): void;
    /**
     * Entries returns the entries of the map, without the expiration time.
     * @returns an iterator over the entries of the map.
     */
    entries(): IterableIterator<[K, V]>;
    /**
     * Values returns the values of the map, without the expiration time.
     * @returns an iterator over the values of the map.
     */
    values(): IterableIterator<V>;
    /**
     * Keys returns the keys of the map
     * @returns an iterator over the keys of the map.
     */
    keys(): IterableIterator<K>;
    /**
     * forEach calls the callbackfn on each entry of the map.
     * @param callbackfn to call on each entry
     * @param thisArg to use as this when calling the callbackfn
     */
    forEach(callbackfn: (value: V, key: K, map: Map<K, V>) => void, thisArg?: ExpirableMap<K, V>): void;
    /**
     * has returns true if the key exists and has not expired.
     * @param key K
     * @returns true if the key exists and has not expired.
     */
    has(key: K): boolean;
    /**
     * delete the entry for the given key.
     * @param key K
     * @returns true if the key existed and has been deleted.
     */
    delete(key: K): boolean;
    /**
     * get size of the map.
     * @returns the size of the map.
     */
    get size(): number;
}
