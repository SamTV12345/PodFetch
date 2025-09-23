import {t} from "i18next";

export class APIError extends Error {
    details: {
        errorCode: string;
        arguments?: Record<string, string>;
    }

    constructor(details: { errorCode: string; arguments?: Record<string, string>; } = {errorCode: "UNKNOWN_ERROR"}) {
        super();
        this.details = details;
    }

    toString() {
        return t(this.details.errorCode, this.details.arguments);
    }
}