import { describe, expect, it } from 'vitest'
import { resolveGatewayUrl } from '#/lib/gateway/gateway-url'

describe('resolveGatewayUrl', () => {
	it('turns an http API origin into a ws gateway url', () => {
		expect(
			resolveGatewayUrl('http://localhost:3456', 'https://app.example'),
		).toBe('ws://localhost:3456/api/v1/chat/gateway')
	})

	it('turns an https API origin into a wss gateway url', () => {
		expect(
			resolveGatewayUrl('https://api.mestier.io', 'https://app.example'),
		).toBe('wss://api.mestier.io/api/v1/chat/gateway')
	})

	it('falls back to the page origin when no api url is configured', () => {
		expect(resolveGatewayUrl('', 'https://app.example')).toBe(
			'wss://app.example/api/v1/chat/gateway',
		)
		expect(resolveGatewayUrl(undefined, 'http://localhost:3000')).toBe(
			'ws://localhost:3000/api/v1/chat/gateway',
		)
	})
})
