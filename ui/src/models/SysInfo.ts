import {CPUModel} from "./CPUModel";
import {SysUser} from "./SysUser";
import {DiskModel} from "./DiskModel";

export interface SysInfo {
    IS_SUPPORTED: boolean,
    SUPPORTED_SIGNALS: string[],
    MINIMUM_CPU_UPDATE_INTERVAL: {
        secs: number,
        nanos: number
    },
    global_cpu_info: {
        cpu_usage: number,
        name: string,
        vendor_id: string,
        brand: string,
        frequency: number,
    },
    cpus: CPUModel[],
    physical_core_count: number,
    total_memory: number,
    free_memory: number,
    used_memory: number,
    total_swap: number,
    free_swap: number,
    used_swap: number,
    components: [],
    users: SysUser[],
    os_version: string,
    long_os_version: string,
    name: string,
    kernel_version: string,
    distribution_id: string,
    host_name: string,

}


export interface ConfigModel {
    podindexConfigured: boolean,
    rssFeed: string
    serverUrl: string,
    basicAuth: string,
    oidcConfigured: boolean,
    oidcConfig?: {
        authority: string,
        clientId: string,
        redirectUri: string,
        scope: string,
    },
    reverseProxy: boolean
}
