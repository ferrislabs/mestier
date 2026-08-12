import type { User } from 'oidc-client-ts'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { onSigninCallback } from '#/components/config-gate'

/** `window.location` itself can't be spied on piecemeal in jsdom — replace
 * the whole getter with a fixture carrying its own `assign` spy. */
function mockLocation(pathname: string) {
	const assign = vi.fn()
	vi.spyOn(window, 'location', 'get').mockReturnValue({
		pathname,
		assign,
	} as unknown as Location)
	return assign
}

describe('onSigninCallback', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('hard-navigates to the state path when it differs from where the IdP landed', () => {
		const assign = mockLocation('/')
		const replaceState = vi.spyOn(window.history, 'replaceState')

		onSigninCallback({ state: '/invite/abc123' } as User)

		expect(assign).toHaveBeenCalledWith('/invite/abc123')
		expect(replaceState).not.toHaveBeenCalled()
	})

	it('strips the query params in place when there is nothing to restore', () => {
		const assign = mockLocation('/')
		const replaceState = vi.spyOn(window.history, 'replaceState')

		onSigninCallback(undefined)

		expect(assign).not.toHaveBeenCalled()
		expect(replaceState).toHaveBeenCalledWith({}, document.title, '/')
	})

	it('ignores a non-string state (opaque application data it does not own)', () => {
		const assign = mockLocation('/')

		onSigninCallback({ state: { unexpected: true } } as unknown as User)

		expect(assign).not.toHaveBeenCalled()
	})
})
