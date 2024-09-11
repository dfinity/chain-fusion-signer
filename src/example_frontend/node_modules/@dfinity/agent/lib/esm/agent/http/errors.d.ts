import { HttpDetailsResponse } from '../api';
export declare class AgentHTTPResponseError extends Error {
    readonly response: HttpDetailsResponse;
    constructor(message: string, response: HttpDetailsResponse);
}
