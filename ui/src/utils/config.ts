import type { components } from '../../schema'

export const getConfigFromHtmlFile = ():
	| components['schemas']['ConfigModel']
	| undefined => {
	const config = document.getElementById('config')

	const dataJson = config?.getAttribute('data-config')

	let configObj: components['schemas']['ConfigModel'] | undefined

	if (dataJson) {
		configObj = JSON.parse(dataJson)
	}
	return configObj
}
