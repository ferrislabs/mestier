const GATEWAY_PATH = '/api/v1/chat/gateway'

/**
 * Derives the gateway WebSocket URL from the API origin (`window.apiUrl`,
 * set by `loadRuntimeConfig`). Falls back to the page's own origin when no
 * API URL is configured — same-origin deployments leave it unset.
 */
export function resolveGatewayUrl(
	apiUrl: string | undefined,
	pageOrigin: string,
): string {
	const origin = apiUrl && apiUrl.length > 0 ? apiUrl : pageOrigin
	const wsProtocol = origin.startsWith('https') ? 'wss' : 'ws'
	const withoutProtocol = origin.replace(/^https?:\/\//, '')
	return `${wsProtocol}://${withoutProtocol}${GATEWAY_PATH}`
}
