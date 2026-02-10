import {paths} from "@/schema";
import createClient from "openapi-react-query";
import createFetchClient, { Middleware } from "openapi-fetch";
import { useStore } from "@/store/store";

const fetchClient = createFetchClient<paths>({
    baseUrl: "", // Will be set dynamically
});

// Middleware to inject the current base URL
const baseUrlMiddleware: Middleware = {
    async onRequest({ request }) {
        const serverUrl = useStore.getState().serverUrl;
        if (serverUrl) {
            // Get the pathname from the original request
            // The request.url might be relative (e.g., "/api/v1/podcasts") or have a placeholder base
            let pathname: string;
            try {
                const url = new URL(request.url);
                pathname = url.pathname + url.search;
            } catch {
                // If URL parsing fails, the request.url is likely a relative path
                pathname = request.url;
            }

            // Combine server URL with the pathname
            const fullUrl = serverUrl.replace(/\/$/, '') + pathname;
            return new Request(fullUrl, request);
        }
        return request;
    },
};

fetchClient.use(baseUrlMiddleware);

export const $api = createClient(fetchClient);


// Function to validate if a PodFetch server is running at the given URL
export const validatePodFetchServer = async (url: string): Promise<boolean> => {
    try {
        // Normalize URL
        let normalizedUrl = url.trim();
        if (!normalizedUrl.startsWith('http://') && !normalizedUrl.startsWith('https://')) {
            normalizedUrl = 'http://' + normalizedUrl;
        }
        // Remove trailing slash
        normalizedUrl = normalizedUrl.replace(/\/$/, '');

        const response = await fetch(`${normalizedUrl}/api/v1/sys/config`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
            },
        });

        // If we get a response (even 401 unauthorized), it's likely a PodFetch server
        return response.ok || response.status === 401;
    } catch (error) {
        console.error('Server validation failed:', error);
        return false;
    }
};

