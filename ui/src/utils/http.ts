import createClient, {Middleware} from "openapi-fetch";
import createTanstackQueryClient from "openapi-react-query";
import type {paths} from "../../schema";
import {APIError} from "./ErrorDefinition";
import { enqueueSnackbar } from "notistack";
import i18n from "../language/i18n";


export let apiURL: string
export let uiURL: string
if (window.location.pathname.startsWith("/ui")) {
    apiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port
} else {
    //match everything before /ui
    const regex = /\/([^/]+)\/ui\//
    apiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port + "/" + regex.exec(window.location.href)![1]
}
uiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port + "/ui"

export const client = createClient<paths>({ baseUrl: apiURL });


export const HEADER_TO_USE: Record<string, string> = {
    "Content-Type": "application/json"
}




export const addHeader = (key: string, value: string) => {
    HEADER_TO_USE[key] = value
}

localStorage.getItem("auth") !== null && addHeader("Authorization", "Basic " + localStorage.getItem("auth"))
sessionStorage.getItem("auth") !== null && addHeader("Authorization", "Basic " + sessionStorage.getItem("auth"))

function isJsonString(str: string) {
    try {
        JSON.parse(str);
    } catch (e) {
        return false;
    }
    return true;
}

const authMiddleware: Middleware = {
    async onRequest({ request}) {
        Object.entries(HEADER_TO_USE).forEach(([key, value]) => {
            request.headers.set(key, value)
        })
        return request;
    },
    async onResponse({ response }) {
        if (!response.ok) {
            if (response.body != null) {
                const textData = await response.text()
                if (isJsonString(textData)) {
                    const e = JSON.parse(textData)
                    // @ts-ignore
                    enqueueSnackbar(i18n.t(e.errorCode, e.arguments), {variant: 'error'})
                    throw new APIError(e)
                } else {
                    throw new Error("Request failed: " + response.body === null? response.statusText: await response.text());
                }
            }
        }
        return response;
    },
};

client.use(authMiddleware)

export const $api = createTanstackQueryClient(client);



client.GET("/api/v1/sys/config", {
    headers: {
        "Content-Type": "application/json"
    },
    asdamasld: {

    }
})
