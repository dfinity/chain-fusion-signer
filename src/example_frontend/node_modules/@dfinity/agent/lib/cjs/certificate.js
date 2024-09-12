"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.check_canister_ranges = exports.flatten_forks = exports.lookup_path = exports.reconstruct = exports.lookupResultToBuffer = exports.Certificate = exports.hashTreeToString = exports.NodeId = exports.CertificateVerificationError = void 0;
const cbor = __importStar(require("./cbor"));
const errors_1 = require("./errors");
const request_id_1 = require("./request_id");
const buffer_1 = require("./utils/buffer");
const principal_1 = require("@dfinity/principal");
const bls = __importStar(require("./utils/bls"));
const leb_1 = require("./utils/leb");
/**
 * A certificate may fail verification with respect to the provided public key
 */
class CertificateVerificationError extends errors_1.AgentError {
    constructor(reason) {
        super(`Invalid certificate: ${reason}`);
    }
}
exports.CertificateVerificationError = CertificateVerificationError;
const NodeId = {
    Empty: 0,
    Fork: 1,
    Labeled: 2,
    Leaf: 3,
    Pruned: 4,
};
exports.NodeId = NodeId;
/**
 * Make a human readable string out of a hash tree.
 * @param tree
 */
function hashTreeToString(tree) {
    const indent = (s) => s
        .split('\n')
        .map(x => '  ' + x)
        .join('\n');
    function labelToString(label) {
        const decoder = new TextDecoder(undefined, { fatal: true });
        try {
            return JSON.stringify(decoder.decode(label));
        }
        catch (e) {
            return `data(...${label.byteLength} bytes)`;
        }
    }
    switch (tree[0]) {
        case NodeId.Empty:
            return '()';
        case NodeId.Fork: {
            if (tree[1] instanceof Array && tree[2] instanceof ArrayBuffer) {
                const left = hashTreeToString(tree[1]);
                const right = hashTreeToString(tree[2]);
                return `sub(\n left:\n${indent(left)}\n---\n right:\n${indent(right)}\n)`;
            }
            else {
                throw new Error('Invalid tree structure for fork');
            }
        }
        case NodeId.Labeled: {
            if (tree[1] instanceof ArrayBuffer && tree[2] instanceof ArrayBuffer) {
                const label = labelToString(tree[1]);
                const sub = hashTreeToString(tree[2]);
                return `label(\n label:\n${indent(label)}\n sub:\n${indent(sub)}\n)`;
            }
            else {
                throw new Error('Invalid tree structure for labeled');
            }
        }
        case NodeId.Leaf: {
            if (!tree[1]) {
                throw new Error('Invalid tree structure for leaf');
            }
            else if (Array.isArray(tree[1])) {
                return JSON.stringify(tree[1]);
            }
            return `leaf(...${tree[1].byteLength} bytes)`;
        }
        case NodeId.Pruned: {
            if (!tree[1]) {
                throw new Error('Invalid tree structure for pruned');
            }
            else if (Array.isArray(tree[1])) {
                return JSON.stringify(tree[1]);
            }
            return `pruned(${(0, buffer_1.toHex)(new Uint8Array(tree[1]))}`;
        }
        default: {
            return `unknown(${JSON.stringify(tree[0])})`;
        }
    }
}
exports.hashTreeToString = hashTreeToString;
function isBufferEqual(a, b) {
    if (a.byteLength !== b.byteLength) {
        return false;
    }
    const a8 = new Uint8Array(a);
    const b8 = new Uint8Array(b);
    for (let i = 0; i < a8.length; i++) {
        if (a8[i] !== b8[i]) {
            return false;
        }
    }
    return true;
}
class Certificate {
    constructor(certificate, _rootKey, _canisterId, _blsVerify, 
    // Default to 5 minutes
    _maxAgeInMinutes = 5) {
        this._rootKey = _rootKey;
        this._canisterId = _canisterId;
        this._blsVerify = _blsVerify;
        this._maxAgeInMinutes = _maxAgeInMinutes;
        this.cert = cbor.decode(new Uint8Array(certificate));
    }
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
    static async create(options) {
        let blsVerify = options.blsVerify;
        if (!blsVerify) {
            blsVerify = bls.blsVerify;
        }
        const cert = new Certificate(options.certificate, options.rootKey, options.canisterId, blsVerify, options.maxAgeInMinutes);
        await cert.verify();
        return cert;
    }
    lookup(path) {
        // constrain the type of the result, so that empty HashTree is undefined
        return lookupResultToBuffer(lookup_path(path, this.cert.tree));
    }
    lookup_label(label) {
        return this.lookup([label]);
    }
    async verify() {
        const rootHash = await reconstruct(this.cert.tree);
        const derKey = await this._checkDelegationAndGetKey(this.cert.delegation);
        const sig = this.cert.signature;
        const key = extractDER(derKey);
        const msg = (0, buffer_1.concat)(domain_sep('ic-state-root'), rootHash);
        let sigVer = false;
        const lookupTime = this.lookup(['time']);
        if (!lookupTime) {
            // Should never happen - time is always present in IC certificates
            throw new CertificateVerificationError('Certificate does not contain a time');
        }
        const FIVE_MINUTES_IN_MSEC = 5 * 60 * 1000;
        const MAX_AGE_IN_MSEC = this._maxAgeInMinutes * 60 * 1000;
        const now = Date.now();
        const earliestCertificateTime = now - MAX_AGE_IN_MSEC;
        const fiveMinutesFromNow = now + FIVE_MINUTES_IN_MSEC;
        const certTime = (0, leb_1.decodeTime)(lookupTime);
        if (certTime.getTime() < earliestCertificateTime) {
            throw new CertificateVerificationError(`Certificate is signed more than ${this._maxAgeInMinutes} minutes in the past. Certificate time: ` +
                certTime.toISOString() +
                ' Current time: ' +
                new Date(now).toISOString());
        }
        else if (certTime.getTime() > fiveMinutesFromNow) {
            throw new CertificateVerificationError('Certificate is signed more than 5 minutes in the future. Certificate time: ' +
                certTime.toISOString() +
                ' Current time: ' +
                new Date(now).toISOString());
        }
        try {
            sigVer = await this._blsVerify(new Uint8Array(key), new Uint8Array(sig), new Uint8Array(msg));
        }
        catch (err) {
            sigVer = false;
        }
        if (!sigVer) {
            throw new CertificateVerificationError('Signature verification failed');
        }
    }
    async _checkDelegationAndGetKey(d) {
        if (!d) {
            return this._rootKey;
        }
        const cert = await Certificate.create({
            certificate: d.certificate,
            rootKey: this._rootKey,
            canisterId: this._canisterId,
            blsVerify: this._blsVerify,
            // Do not check max age for delegation certificates
            maxAgeInMinutes: Infinity,
        });
        const canisterInRange = check_canister_ranges({
            canisterId: this._canisterId,
            subnetId: principal_1.Principal.fromUint8Array(new Uint8Array(d.subnet_id)),
            tree: cert.cert.tree,
        });
        if (!canisterInRange) {
            throw new CertificateVerificationError(`Canister ${this._canisterId} not in range of delegations for subnet 0x${(0, buffer_1.toHex)(d.subnet_id)}`);
        }
        const publicKeyLookup = cert.lookup(['subnet', d.subnet_id, 'public_key']);
        if (!publicKeyLookup) {
            throw new Error(`Could not find subnet key for subnet 0x${(0, buffer_1.toHex)(d.subnet_id)}`);
        }
        return publicKeyLookup;
    }
}
exports.Certificate = Certificate;
const DER_PREFIX = (0, buffer_1.fromHex)('308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100');
const KEY_LENGTH = 96;
function extractDER(buf) {
    const expectedLength = DER_PREFIX.byteLength + KEY_LENGTH;
    if (buf.byteLength !== expectedLength) {
        throw new TypeError(`BLS DER-encoded public key must be ${expectedLength} bytes long`);
    }
    const prefix = buf.slice(0, DER_PREFIX.byteLength);
    if (!isBufferEqual(prefix, DER_PREFIX)) {
        throw new TypeError(`BLS DER-encoded public key is invalid. Expect the following prefix: ${DER_PREFIX}, but get ${prefix}`);
    }
    return buf.slice(DER_PREFIX.byteLength);
}
/**
 * utility function to constrain the type of a path
 * @param {ArrayBuffer | HashTree | undefined} result - the result of a lookup
 * @returns ArrayBuffer or Undefined
 */
function lookupResultToBuffer(result) {
    if (result instanceof ArrayBuffer) {
        return result;
    }
    else if (result instanceof Uint8Array) {
        return result.buffer;
    }
    return undefined;
}
exports.lookupResultToBuffer = lookupResultToBuffer;
/**
 * @param t
 */
async function reconstruct(t) {
    switch (t[0]) {
        case NodeId.Empty:
            return (0, request_id_1.hash)(domain_sep('ic-hashtree-empty'));
        case NodeId.Pruned:
            return t[1];
        case NodeId.Leaf:
            return (0, request_id_1.hash)((0, buffer_1.concat)(domain_sep('ic-hashtree-leaf'), t[1]));
        case NodeId.Labeled:
            return (0, request_id_1.hash)((0, buffer_1.concat)(domain_sep('ic-hashtree-labeled'), t[1], await reconstruct(t[2])));
        case NodeId.Fork:
            return (0, request_id_1.hash)((0, buffer_1.concat)(domain_sep('ic-hashtree-fork'), await reconstruct(t[1]), await reconstruct(t[2])));
        default:
            throw new Error('unreachable');
    }
}
exports.reconstruct = reconstruct;
function domain_sep(s) {
    const len = new Uint8Array([s.length]);
    const str = new TextEncoder().encode(s);
    return (0, buffer_1.concat)(len, str);
}
/**
 * @param path
 * @param tree
 */
function lookup_path(path, tree) {
    if (path.length === 0) {
        switch (tree[0]) {
            case NodeId.Leaf: {
                // should not be undefined
                if (!tree[1])
                    throw new Error('Invalid tree structure for leaf');
                if (tree[1] instanceof ArrayBuffer) {
                    return tree[1];
                }
                else if (tree[1] instanceof Uint8Array) {
                    return tree[1].buffer;
                }
                else
                    return tree[1];
            }
            case NodeId.Fork: {
                return tree;
            }
            default: {
                return tree;
            }
        }
    }
    const label = typeof path[0] === 'string' ? new TextEncoder().encode(path[0]) : path[0];
    const t = find_label(label, flatten_forks(tree));
    if (t) {
        return lookup_path(path.slice(1), t);
    }
}
exports.lookup_path = lookup_path;
/**
 * If the tree is a fork, flatten it into an array of trees
 * @param t - the tree to flatten
 * @returns HashTree[] - the flattened tree
 */
function flatten_forks(t) {
    switch (t[0]) {
        case NodeId.Empty:
            return [];
        case NodeId.Fork:
            return flatten_forks(t[1]).concat(flatten_forks(t[2]));
        default:
            return [t];
    }
}
exports.flatten_forks = flatten_forks;
function find_label(l, trees) {
    if (trees.length === 0) {
        return undefined;
    }
    for (const t of trees) {
        if (t[0] === NodeId.Labeled) {
            const p = t[1];
            if (isBufferEqual(l, p)) {
                return t[2];
            }
        }
    }
}
/**
 * Check if a canister falls within a range of canisters
 * @param canisterId Principal
 * @param ranges [Principal, Principal][]
 * @returns
 */
function check_canister_ranges(params) {
    const { canisterId, subnetId, tree } = params;
    const rangeLookup = lookup_path(['subnet', subnetId.toUint8Array(), 'canister_ranges'], tree);
    if (!rangeLookup || !(rangeLookup instanceof ArrayBuffer)) {
        throw new Error(`Could not find canister ranges for subnet ${subnetId}`);
    }
    const ranges_arr = cbor.decode(rangeLookup);
    const ranges = ranges_arr.map(v => [
        principal_1.Principal.fromUint8Array(v[0]),
        principal_1.Principal.fromUint8Array(v[1]),
    ]);
    const canisterInRange = ranges.some(r => r[0].ltEq(canisterId) && r[1].gtEq(canisterId));
    return canisterInRange;
}
exports.check_canister_ranges = check_canister_ranges;
//# sourceMappingURL=certificate.js.map