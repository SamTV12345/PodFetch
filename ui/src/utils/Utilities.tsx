export const isLocalhost = Boolean(
    window.location.hostname === 'localhost' ||
    // [::1] is the IPv6 localhost address.
    window.location.hostname === '[::1]' ||
    // 127.0.0.0/8 are considered localhost for IPv4.
    window.location.hostname.match(
        /^127(?:\.(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}$/
    )
);

export let apiURL=''
export let uiURL=''

if(isLocalhost){
    apiURL="http://localhost:8000/api/v1"
    uiURL="http://localhost:5173/ui"
}
else {
    apiURL=window.location.protocol+"//"+window.location.hostname+"/api"
    uiURL=window.location.protocol+"//"+window.location.hostname+"/ui"
}
