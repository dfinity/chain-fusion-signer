"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AgentHTTPResponseError = void 0;
class AgentHTTPResponseError extends Error {
    constructor(message, response) {
        super(message);
        this.response = response;
        this.name = this.constructor.name;
        Object.setPrototypeOf(this, new.target.prototype);
    }
}
exports.AgentHTTPResponseError = AgentHTTPResponseError;
//# sourceMappingURL=errors.js.map