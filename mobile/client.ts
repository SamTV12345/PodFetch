import {paths} from "@/schema";
import createClient from "openapi-react-query";
import createFetchClient from "openapi-fetch";

const fetchClient = createFetchClient<paths>({
    baseUrl: "http://localhost:8000",
});

export const $api = createClient(fetchClient);

export const HEADER_TO_USE: Record<string, string> = {
    "Content-Type": "application/json"
}




export const addHeader = (key: string, value: string) => {
    HEADER_TO_USE[key] = value
}