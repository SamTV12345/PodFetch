import { enqueueSnackbar } from 'notistack'
import createClient, { type Middleware } from 'openapi-fetch'
import createTanstackQueryClient from 'openapi-react-query'
import type { paths } from '../../schema'
import i18n from '../language/i18n'
import { getConfigFromHtmlFile } from './config'
import { APIError } from './ErrorDefinition'
import { getLogin } from './login'

export let apiURL: string
export const uiURL: string =
	window.location.protocol +
	'//' +
	window.location.hostname +
	':' +
	window.location.port +
	'/ui'
if (window.location.pathname.startsWith('/ui')) {
	apiURL =
		window.location.protocol +
		'//' +
		window.location.hostname +
		':' +
		window.location.port
} else {
	//match everything before /ui
	const regex = /\/([^/]+)\/ui\//
	apiURL =
		window.location.protocol +
		'//' +
		window.location.hostname +
		':' +
		window.location.port +
		'/' +
		regex.exec(window.location.href)?.[1]
}

export const client = createClient<paths>({ baseUrl: apiURL })

export const HEADER_TO_USE: Record<string, string> = {
	'Content-Type': 'application/json',
}

const configObj = getConfigFromHtmlFile()

function isJsonString(str: string) {
	try {
		JSON.parse(str)
	} catch (_e) {
		return false
	}
	return true
}

const authMiddleware: Middleware = {
	async onRequest({ request }) {
		const auth = localStorage.getItem('auth') || sessionStorage.getItem('auth')
		const _login = getLogin()
		Object.entries(HEADER_TO_USE).forEach(([key, value]) => {
			request.headers.set(key, value)
		})
		if (auth && configObj && configObj.basicAuth) {
			request.headers.set('Authorization', `Basic ${auth}`)
		} else if (auth && configObj && configObj.oidcConfigured) {
			request.headers.set('Authorization', `Bearer ${auth}`)
		}
		return request
	},
	async onResponse({ response }) {
		if (!response.ok) {
			if (response.body != null) {
				const textData = await response.text()
				if (isJsonString(textData)) {
					const e = JSON.parse(textData)
					// @ts-expect-error
					enqueueSnackbar(i18n.t(e.errorCode, e.arguments), {
						variant: 'error',
					})
					throw new APIError(e)
				} else {
					throw new Error(
						`Request failed: ${response.body}` === null
							? response.statusText
							: textData,
					)
				}
			}
		}
		return response
	},
}

client.use(authMiddleware)

export const $api = createTanstackQueryClient(client)
