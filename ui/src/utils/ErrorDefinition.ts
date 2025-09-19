export class APIError extends Error {
    details: {
        errorCode: string;
        arguments?: Record<string, string>;
    }

    constructor(details: { errorCode: string; arguments?: Record<string, string>; } = {errorCode: "UNKNOWN_ERROR"}) {
        super();
        this.details = details;
    }
}