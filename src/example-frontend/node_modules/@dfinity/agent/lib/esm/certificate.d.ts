import { AgentError } from './errors';
import { Principal } from '@dfinity/principal';
/**
 * A certificate may fail verification with respect to the provided public key
 */
export declare class CertificateVerificationError extends AgentError {
    constructor(reason: string);
}
export interface Cert {
    tree: HashTree;
    signature: ArrayBuffer;
    delegation?: Delegation;
}
declare const NodeId: {
    Empty: number;
    Fork: number;
    Labeled: number;
    Leaf: number;
    Pruned: number;
};
export declare type NodeIdType = typeof NodeId[keyof typeof NodeId];
export { NodeId };
export declare type HashTree = [typeof NodeId.Empty] | [typeof NodeId.Fork, HashTree, HashTree] | [typeof NodeId.Labeled, ArrayBuffer, HashTree] | [typeof NodeId.Leaf, ArrayBuffer] | [typeof NodeId.Pruned, ArrayBuffer];
/**
 * Make a human readable string out of a hash tree.
 * @param tree
 */
export declare function hashTreeToString(tree: HashTree): string;
interface Delegation extends Record<string, any> {
    subnet_id: ArrayBuffer;
    certificate: ArrayBuffer;
}
declare type VerifyFunc = (pk: Uint8Array, sig: Uint8Array, msg: Uint8Array) => Promise<boolean>;
export interface CreateCertificateOptions {
    /**
     * The bytes encoding the certificate to be verified
     */
    certificate: ArrayBuffer;
    /**
     * The root key against which to verify the certificate
     * (normally, the root key of the IC main network)
     */
    rootKey: ArrayBuffer;
    /**
     * The effective canister ID of the request when verifying a response, or
     * the signing canister ID when verifying a certified variable.
     */
    canisterId: Principal;
    /**
     * BLS Verification strategy. Default strategy uses wasm for performance, but that may not be available in all contexts.
     */
    blsVerify?: VerifyFunc;
    /**
     * The maximum age of the certificate in minutes. Default is 5 minutes.
     * @default 5
     * This is used to verify the time the certificate was signed, particularly for validating Delegation certificates, which can live for longer than the default window of +/- 5 minutes. If the certificate is
     * older than the specified age, it will fail verification.
     */
    maxAgeInMinutes?: number;
}
export declare class Certificate {
    private _rootKey;
    private _canisterId;
    private _blsVerify;
    private _maxAgeInMinutes;
    private readonly cert;
    /**
     * Create a new instance of a certificate, automatically verifying it. Throws a
     * CertificateVerificationError if the certificate cannot be verified.
     * @constructs  Certificate
     * @param {CreateCertificateOptions} options {@link CreateCertificateOptions}
     * @param {ArrayBuffer} options.certificate The bytes of the certificate
     * @param {ArrayBuffer} options.rootKey The root key to verify against
     * @param {Principal} options.canisterId The effective or signing canister ID
     * @param {number} options.maxAgeInMinutes The maximum age of the certificate in minutes. Default is 5 minutes.
     * @throws {CertificateVerificationError}
     */
    static create(options: CreateCertificateOptions): Promise<Certificate>;
    private constructor();
    lookup(path: Array<ArrayBuffer | string>): ArrayBuffer | undefined;
    lookup_label(label: ArrayBuffer): ArrayBuffer | HashTree | undefined;
    private verify;
    private _checkDelegationAndGetKey;
}
/**
 * utility function to constrain the type of a path
 * @param {ArrayBuffer | HashTree | undefined} result - the result of a lookup
 * @returns ArrayBuffer or Undefined
 */
export declare function lookupResultToBuffer(result: ArrayBuffer | HashTree | undefined): ArrayBuffer | undefined;
/**
 * @param t
 */
export declare function reconstruct(t: HashTree): Promise<ArrayBuffer>;
/**
 * @param path
 * @param tree
 */
export declare function lookup_path(path: Array<ArrayBuffer | string>, tree: HashTree): ArrayBuffer | HashTree | undefined;
/**
 * If the tree is a fork, flatten it into an array of trees
 * @param t - the tree to flatten
 * @returns HashTree[] - the flattened tree
 */
export declare function flatten_forks(t: HashTree): HashTree[];
/**
 * Check if a canister falls within a range of canisters
 * @param canisterId Principal
 * @param ranges [Principal, Principal][]
 * @returns
 */
export declare function check_canister_ranges(params: {
    canisterId: Principal;
    subnetId: Principal;
    tree: HashTree;
}): boolean;
