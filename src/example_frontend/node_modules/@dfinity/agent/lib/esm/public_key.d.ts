import { DerEncodedPublicKey, PublicKey } from './auth';
export declare class Ed25519PublicKey implements PublicKey {
    #private;
    static from(key: PublicKey): Ed25519PublicKey;
    static fromRaw(rawKey: ArrayBuffer): Ed25519PublicKey;
    static fromDer(derKey: DerEncodedPublicKey): Ed25519PublicKey;
    private static RAW_KEY_LENGTH;
    private static derEncode;
    private static derDecode;
    get rawKey(): ArrayBuffer;
    get derKey(): DerEncodedPublicKey;
    private constructor();
    toDer(): DerEncodedPublicKey;
    toRaw(): ArrayBuffer;
}
