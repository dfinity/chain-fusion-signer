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
exports.encodePath = exports.fetchNodeKeys = exports.request = void 0;
/** @module CanisterStatus */
const principal_1 = require("@dfinity/principal");
const errors_1 = require("../errors");
const certificate_1 = require("../certificate");
const buffer_1 = require("../utils/buffer");
const Cbor = __importStar(require("../cbor"));
const leb_1 = require("../utils/leb");
/**
 *
 * @param {CanisterStatusOptions} options {@link CanisterStatusOptions}
 * @param {CanisterStatusOptions['canisterId']} options.canisterId {@link Principal}
 * @param {CanisterStatusOptions['agent']} options.agent {@link HttpAgent} optional authenticated agent to use to make the canister request. Useful for accessing private metadata under icp:private
 * @param {CanisterStatusOptions['paths']} options.paths {@link Path[]}
 * @returns {Status} object populated with data from the requested paths
 * @example
 * const status = await canisterStatus({
 *   paths: ['controllers', 'candid'],
 *   ...options
 * });
 *
 * const controllers = status.get('controllers');
 */
const request = async (options) => {
    const { agent, paths } = options;
    const canisterId = principal_1.Principal.from(options.canisterId);
    const uniquePaths = [...new Set(paths)];
    // Map path options to their correct formats
    const encodedPaths = uniquePaths.map(path => {
        return (0, exports.encodePath)(path, canisterId);
    });
    const status = new Map();
    const promises = uniquePaths.map((path, index) => {
        return (async () => {
            var _a;
            try {
                const response = await agent.readState(canisterId, {
                    paths: [encodedPaths[index]],
                });
                const cert = await certificate_1.Certificate.create({
                    certificate: response.certificate,
                    rootKey: agent.rootKey,
                    canisterId: canisterId,
                });
                const lookup = (cert, path) => {
                    if (path === 'subnet') {
                        const data = (0, exports.fetchNodeKeys)(response.certificate, canisterId, agent.rootKey);
                        return {
                            path: path,
                            data,
                        };
                    }
                    else {
                        return {
                            path: path,
                            data: (0, certificate_1.lookupResultToBuffer)(cert.lookup((0, exports.encodePath)(path, canisterId))),
                        };
                    }
                };
                // must pass in the rootKey if we have no delegation
                const { path, data } = lookup(cert, uniquePaths[index]);
                if (!data) {
                    // Typically, the cert lookup will throw
                    console.warn(`Expected to find result for path ${path}, but instead found nothing.`);
                    if (typeof path === 'string') {
                        status.set(path, null);
                    }
                    else {
                        status.set(path.key, null);
                    }
                }
                else {
                    switch (path) {
                        case 'time': {
                            status.set(path, (0, leb_1.decodeTime)(data));
                            break;
                        }
                        case 'controllers': {
                            status.set(path, decodeControllers(data));
                            break;
                        }
                        case 'module_hash': {
                            status.set(path, decodeHex(data));
                            break;
                        }
                        case 'subnet': {
                            status.set(path, data);
                            break;
                        }
                        case 'candid': {
                            status.set(path, new TextDecoder().decode(data));
                            break;
                        }
                        default: {
                            // Check for CustomPath signature
                            if (typeof path !== 'string' && 'key' in path && 'path' in path) {
                                switch (path.decodeStrategy) {
                                    case 'raw':
                                        status.set(path.key, data);
                                        break;
                                    case 'leb128': {
                                        status.set(path.key, (0, leb_1.decodeLeb128)(data));
                                        break;
                                    }
                                    case 'cbor': {
                                        status.set(path.key, decodeCbor(data));
                                        break;
                                    }
                                    case 'hex': {
                                        status.set(path.key, decodeHex(data));
                                        break;
                                    }
                                    case 'utf-8': {
                                        status.set(path.key, decodeUtf8(data));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            catch (error) {
                // Break on signature verification errors
                if ((_a = error === null || error === void 0 ? void 0 : error.message) === null || _a === void 0 ? void 0 : _a.includes('Invalid certificate')) {
                    throw new errors_1.AgentError(error.message);
                }
                if (typeof path !== 'string' && 'key' in path && 'path' in path) {
                    status.set(path.key, null);
                }
                else {
                    status.set(path, null);
                }
                console.group();
                console.warn(`Expected to find result for path ${path}, but instead found nothing.`);
                console.warn(error);
                console.groupEnd();
            }
        })();
    });
    // Fetch all values separately, as each option can fail
    await Promise.all(promises);
    return status;
};
exports.request = request;
const fetchNodeKeys = (certificate, canisterId, root_key) => {
    if (!canisterId._isPrincipal) {
        throw new Error('Invalid canisterId');
    }
    const cert = Cbor.decode(new Uint8Array(certificate));
    const tree = cert.tree;
    let delegation = cert.delegation;
    let subnetId;
    if (delegation && delegation.subnet_id) {
        subnetId = principal_1.Principal.fromUint8Array(new Uint8Array(delegation.subnet_id));
    }
    // On local replica, with System type subnet, there is no delegation
    else if (!delegation && typeof root_key !== 'undefined') {
        subnetId = principal_1.Principal.selfAuthenticating(new Uint8Array(root_key));
        delegation = {
            subnet_id: subnetId.toUint8Array(),
            certificate: new ArrayBuffer(0),
        };
    }
    // otherwise use default NNS subnet id
    else {
        subnetId = principal_1.Principal.selfAuthenticating(principal_1.Principal.fromText('tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe').toUint8Array());
        delegation = {
            subnet_id: subnetId.toUint8Array(),
            certificate: new ArrayBuffer(0),
        };
    }
    const canisterInRange = (0, certificate_1.check_canister_ranges)({ canisterId, subnetId, tree });
    if (!canisterInRange) {
        throw new Error('Canister not in range');
    }
    const nodeTree = (0, certificate_1.lookup_path)(['subnet', delegation === null || delegation === void 0 ? void 0 : delegation.subnet_id, 'node'], tree);
    const nodeForks = (0, certificate_1.flatten_forks)(nodeTree);
    nodeForks.length;
    const nodeKeys = new Map();
    nodeForks.forEach(fork => {
        Object.getPrototypeOf(new Uint8Array(fork[1]));
        const node_id = principal_1.Principal.from(new Uint8Array(fork[1])).toText();
        const derEncodedPublicKey = (0, certificate_1.lookup_path)(['public_key'], fork[2]);
        if (derEncodedPublicKey.byteLength !== 44) {
            throw new Error('Invalid public key length');
        }
        else {
            nodeKeys.set(node_id, derEncodedPublicKey);
        }
    });
    return {
        subnetId: principal_1.Principal.fromUint8Array(new Uint8Array(delegation.subnet_id)).toText(),
        nodeKeys,
    };
};
exports.fetchNodeKeys = fetchNodeKeys;
const encodePath = (path, canisterId) => {
    const encoder = new TextEncoder();
    const encode = (arg) => {
        return new DataView(encoder.encode(arg).buffer).buffer;
    };
    const canisterBuffer = new DataView(canisterId.toUint8Array().buffer).buffer;
    switch (path) {
        case 'time':
            return [encode('time')];
        case 'controllers':
            return [encode('canister'), canisterBuffer, encode('controllers')];
        case 'module_hash':
            return [encode('canister'), canisterBuffer, encode('module_hash')];
        case 'subnet':
            return [encode('subnet')];
        case 'candid':
            return [encode('canister'), canisterBuffer, encode('metadata'), encode('candid:service')];
        default: {
            // Check for CustomPath signature
            if ('key' in path && 'path' in path) {
                // For simplified metadata queries
                if (typeof path['path'] === 'string' || path['path'] instanceof ArrayBuffer) {
                    const metaPath = path.path;
                    const encoded = typeof metaPath === 'string' ? encode(metaPath) : metaPath;
                    return [encode('canister'), canisterBuffer, encode('metadata'), encoded];
                    // For non-metadata, return the provided custompath
                }
                else {
                    return path['path'];
                }
            }
        }
    }
    throw new Error(`An unexpeected error was encountered while encoding your path for canister status. Please ensure that your path, ${path} was formatted correctly.`);
};
exports.encodePath = encodePath;
const decodeHex = (buf) => {
    return (0, buffer_1.toHex)(buf);
};
const decodeCbor = (buf) => {
    return Cbor.decode(buf);
};
const decodeUtf8 = (buf) => {
    return new TextDecoder().decode(buf);
};
// Controllers are CBOR-encoded buffers
const decodeControllers = (buf) => {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const controllersRaw = decodeCbor(buf);
    return controllersRaw.map((buf) => {
        return principal_1.Principal.fromUint8Array(new Uint8Array(buf));
    });
};
//# sourceMappingURL=index.js.map