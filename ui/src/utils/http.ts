import createClient, {Middleware} from "openapi-fetch";

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

import type { paths } from "../../schema";
import useCommon from "../store/CommonSlice";

export const client = createClient<paths>({ baseUrl: apiURL });


export const HEADER_TO_USE: Record<string, string> = {
    "Content-Type": "application/json"
}




export const addHeader = (key: string, value: string) => {
    HEADER_TO_USE[key] = value
}

localStorage.getItem("auth") !== null && addHeader("Authorization", "Basic " + localStorage.getItem("auth"))
sessionStorage.getItem("auth") !== null && addHeader("Authorization", "Basic " + sessionStorage.getItem("auth"))

const authMiddleware: Middleware = {
    async onRequest({ request}) {

        Object.entries(HEADER_TO_USE).forEach(([key, value]) => {
            request.headers.set(key, value)
        })
        return request;
    },
    async onResponse({ response }) {

        if (!response.ok) {
            throw new Error("Request failed: " + response.body === null? response.statusText: await response.text() );
        }

        return response;
    },
};

client.use(authMiddleware)


client.GET("/api/v1/sys/config", {
    headers: {
        "Content-Type": "application/json"
    },
    asdamasld: {

    }
})
