import { JsonObject } from '@dfinity/candid';
import { Principal } from '@dfinity/principal';
import { AgentError } from '../../errors';
import { Identity } from '../../auth';
import { Agent, ApiQueryResponse, QueryFields, ReadStateOptions, ReadStateResponse, SubmitResponse } from '../api';
import { HttpAgentRequest, HttpAgentRequestTransformFn } from './types';
import { SubnetStatus } from '../../canisterStatus';
export * from './transforms';
export { Nonce, makeNonce } from './types';
export declare enum RequestStatusResponseStatus {
    Received = "received",
    Processing = "processing",
    Replied = "replied",
    Rejected = "rejected",
    Unknown = "unknown",
    Done = "done"
}
export declare class IdentityInvalidError extends AgentError {
    readonly message: string;
    constructor(message: string);
}
export interface HttpAgentOptions {
    source?: HttpAgent;
    fetch?: typeof fetch;
    fetchOptions?: Record<string, unknown>;
    callOptions?: Record<string, unknown>;
    host?: string;
    identity?: Identity | Promise<Identity>;
    credentials?: {
        name: string;
        password?: string;
    };
    /**
     * Adds a unique {@link Nonce} with each query.
     * Enabling will prevent queries from being answered with a cached response.
     *
     * @example
     * const agent = new HttpAgent({ useQueryNonces: true });
     * agent.addTransform(makeNonceTransform(makeNonce);
     * @default false
     */
    useQueryNonces?: boolean;
    /**
     * Number of times to retry requests before throwing an error
     * @default 3
     */
    retryTimes?: number;
    /**
     * Whether the agent should verify signatures signed by node keys on query responses. Increases security, but adds overhead and must make a separate request to cache the node keys for the canister's subnet.
     * @default true
     */
    verifyQuerySignatures?: boolean;
}
export declare class HttpAgent implements Agent {
    #private;
    rootKey: ArrayBuffer;
    private _identity;
    private readonly _fetch;
    private readonly _fetchOptions?;
    private readonly _callOptions?;
    private _timeDiffMsecs;
    private readonly _host;
    private readonly _credentials;
    private _rootKeyFetched;
    private readonly _retryTimes;
    readonly _isAgent = true;
    constructor(options?: HttpAgentOptions);
    isLocal(): boolean;
    addTransform(type: 'update' | 'query', fn: HttpAgentRequestTransformFn, priority?: number): void;
    getPrincipal(): Promise<Principal>;
    call(canisterId: Principal | string, options: {
        methodName: string;
        arg: ArrayBuffer;
        effectiveCanisterId?: Principal | string;
    }, identity?: Identity | Promise<Identity>): Promise<SubmitResponse>;
    private _requestAndRetry;
    query(canisterId: Principal | string, fields: QueryFields, identity?: Identity | Promise<Identity>): Promise<ApiQueryResponse>;
    createReadStateRequest(fields: ReadStateOptions, identity?: Identity | Promise<Identity>): Promise<any>;
    readState(canisterId: Principal | string, fields: ReadStateOptions, identity?: Identity | Promise<Identity>, request?: any): Promise<ReadStateResponse>;
    /**
     * Allows agent to sync its time with the network. Can be called during intialization or mid-lifecycle if the device's clock has drifted away from the network time. This is necessary to set the Expiry for a request
     * @param {Principal} canisterId - Pass a canister ID if you need to sync the time with a particular replica. Uses the management canister by default
     */
    syncTime(canisterId?: Principal): Promise<void>;
    status(): Promise<JsonObject>;
    fetchRootKey(): Promise<ArrayBuffer>;
    invalidateIdentity(): void;
    replaceIdentity(identity: Identity): void;
    fetchSubnetKeys(canisterId: Principal | string): Promise<SubnetStatus | undefined>;
    protected _transform(request: HttpAgentRequest): Promise<HttpAgentRequest>;
}
