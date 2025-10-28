import type { CPUModel } from './CPUModel'
import type { SysUser } from './SysUser'

export interface SysInfo {
	IS_SUPPORTED: boolean
	SUPPORTED_SIGNALS: string[]
	MINIMUM_CPU_UPDATE_INTERVAL: {
		secs: number
		nanos: number
	}
	global_cpu_usage: number
	cpus: CPUModelAggregate
	physical_core_count: number
	mem_total: number
	mem_available: number
	used_memory: number
	total_swap: number
	free_swap: number
	used_swap: number
	components: []
	users: SysUser[]
	os_version: string
	long_os_version: string
	name: string
	kernel_version: string
	distribution_id: string
	host_name: string
}

type CPUModelAggregate = {
	cpus: CPUModel[]
	global: number
}

export interface ConfigModel {
	podindexConfigured: boolean
	rssFeed: string
	serverUrl: string
	wsUrl: string
	basicAuth: string
	oidcConfigured: boolean
	oidcConfig?: {
		authority: string
		clientId: string
		redirectUri: string
		scope: string
	}
	reverseProxy: boolean
}
