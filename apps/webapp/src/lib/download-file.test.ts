import { Blob } from 'node:buffer'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { downloadAuthenticatedFile } from '#/lib/download-file'
import { authStore, clearAuth, setAuth } from '#/store/auth.store'

describe('downloadAuthenticatedFile', () => {
	const createObjectURL = vi.fn(() => 'blob:mestier/fake')
	const revokeObjectURL = vi.fn()

	beforeEach(() => {
		window.apiUrl = 'http://api.test'
		setAuth({ accessToken: 'token-123' })
		URL.createObjectURL = createObjectURL
		URL.revokeObjectURL = revokeObjectURL
	})

	afterEach(() => {
		clearAuth()
		vi.restoreAllMocks()
		createObjectURL.mockClear()
		revokeObjectURL.mockClear()
	})

	it('fetches the API URL with the bearer token attached', async () => {
		const blob = new Blob(['%PDF-1.4'], { type: 'application/pdf' })
		const fetchSpy = vi
			.spyOn(globalThis, 'fetch')
			.mockResolvedValue(new Response(blob, { status: 200 }))

		await downloadAuthenticatedFile(
			'/api/v1/invoices/inv-1/pdf',
			'facture-inv-1.pdf',
		)

		expect(fetchSpy).toHaveBeenCalledTimes(1)
		const [url, init] = fetchSpy.mock.calls[0] as [string, RequestInit]
		expect(url).toBe('http://api.test/api/v1/invoices/inv-1/pdf')
		expect((init.headers as Headers).get('Authorization')).toBe(
			'Bearer token-123',
		)
	})

	it('triggers a browser download and releases the object URL', async () => {
		const blob = new Blob(['%PDF-1.4'], { type: 'application/pdf' })
		vi.spyOn(globalThis, 'fetch').mockResolvedValue(
			new Response(blob, { status: 200 }),
		)
		const clickSpy = vi
			.spyOn(HTMLAnchorElement.prototype, 'click')
			.mockImplementation(() => {})

		await downloadAuthenticatedFile(
			'/api/v1/invoices/inv-1/pdf',
			'facture-inv-1.pdf',
		)

		expect(createObjectURL).toHaveBeenCalledTimes(1)
		expect(clickSpy).toHaveBeenCalledTimes(1)
		expect(revokeObjectURL).toHaveBeenCalledWith('blob:mestier/fake')
	})

	it('omits the Authorization header when there is no access token', async () => {
		authStore.setState((state) => ({ ...state, accessToken: null }))
		const blob = new Blob(['%PDF-1.4'], { type: 'application/pdf' })
		const fetchSpy = vi
			.spyOn(globalThis, 'fetch')
			.mockResolvedValue(new Response(blob, { status: 200 }))

		await downloadAuthenticatedFile('/api/v1/invoices/inv-1/pdf', 'facture.pdf')

		const [, init] = fetchSpy.mock.calls[0] as [string, RequestInit]
		expect((init.headers as Headers).has('Authorization')).toBe(false)
	})

	it('refuses a failed response rather than downloading an error body', async () => {
		vi.spyOn(globalThis, 'fetch').mockResolvedValue(
			new Response('not found', { status: 404, statusText: 'Not Found' }),
		)

		await expect(
			downloadAuthenticatedFile('/api/v1/invoices/missing/pdf', 'facture.pdf'),
		).rejects.toThrow('HTTP 404')
		expect(createObjectURL).not.toHaveBeenCalled()
	})
})
