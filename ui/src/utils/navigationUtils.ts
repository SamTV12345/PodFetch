const wsEndpoint = "ws"

export const configWSUrl = (url: string) => {
    if (url.startsWith("http")) {
        return url.replace("http", "ws") + wsEndpoint
    }
    return url.replace("https", "wss") + wsEndpoint
}
