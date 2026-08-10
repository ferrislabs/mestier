import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { AuthContextProps, ErrorContext } from 'react-oidc-context'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { AuthGate } from '#/components/auth-gate'

const useAuth = vi.hoisted(() => vi.fn())
vi.mock('react-oidc-context', () => ({ useAuth }))
vi.mock('#/components/auth-token-sync', () => ({ AuthTokenSync: () => null }))
vi.mock('#/lib/runtime-config', () => ({
	getOidcConfiguration: () => ({
		authority: 'http://idp.test/realms/mestier',
		client_id: 'mestier-webapp',
		redirect_uri: 'http://app.test/',
		scope: 'openid',
	}),
}))

interface AuthStub {
	isLoading?: boolean
	isAuthenticated?: boolean
	activeNavigator?: AuthContextProps['activeNavigator']
	error?: ErrorContext
}

function mountGate(stub: AuthStub) {
	const removeUser = vi.fn().mockResolvedValue(undefined)
	const signinRedirect = vi.fn().mockResolvedValue(undefined)

	useAuth.mockReturnValue({
		isLoading: false,
		isAuthenticated: false,
		activeNavigator: undefined,
		error: undefined,
		removeUser,
		signinRedirect,
		...stub,
	})

	render(
		<AuthGate>
			<p>Espace de travail</p>
		</AuthGate>,
	)

	return { removeUser, signinRedirect }
}

function errorFrom(source: ErrorContext['source'], message: string) {
	return { name: 'Error', message, source } as ErrorContext
}

const REFRESH_FAILURE = errorFrom(
	'renewSilent',
	'Unauthorized (401): {"code":"E_UNAUTHORIZED","message":"Invalid refresh token"}',
)

describe('AuthGate', () => {
	beforeEach(() => {
		window.sessionStorage.clear()
		useAuth.mockReset()
	})

	it('redirects to the provider when no session exists', async () => {
		const { signinRedirect } = mountGate({})

		await waitFor(() => {
			expect(signinRedirect).toHaveBeenCalledTimes(1)
		})
	})

	it('treats a renewal failure as the end of the session', async () => {
		const { removeUser, signinRedirect } = mountGate({ error: REFRESH_FAILURE })

		await waitFor(() => {
			expect(removeUser).toHaveBeenCalledTimes(1)
		})
		await waitFor(() => {
			expect(signinRedirect).toHaveBeenCalledTimes(1)
		})

		expect(screen.getByText('Session expirée')).toBeDefined()
		expect(screen.queryByText("Erreur d'authentification")).toBeNull()
	})

	it('purges the stale token before signing in again', async () => {
		const order: string[] = []
		const removeUser = vi.fn(async () => {
			order.push('removeUser')
		})
		const signinRedirect = vi.fn(async () => {
			order.push('signinRedirect')
		})

		useAuth.mockReturnValue({
			isLoading: false,
			isAuthenticated: false,
			activeNavigator: undefined,
			error: REFRESH_FAILURE,
			removeUser,
			signinRedirect,
		})
		render(
			<AuthGate>
				<p>Espace de travail</p>
			</AuthGate>,
		)

		await waitFor(() => {
			expect(order).toEqual(['removeUser', 'signinRedirect'])
		})
	})

	it('shows the error without redirecting for a failure that is not a session end', async () => {
		const { signinRedirect } = mountGate({
			error: errorFrom('signinCallback', 'Sign-in failed'),
		})

		expect(screen.getByText("Erreur d'authentification")).toBeDefined()
		expect(screen.getByText('Sign-in failed')).toBeDefined()
		expect(signinRedirect).not.toHaveBeenCalled()
	})

	it('does not retry the reconnection twice in a row', async () => {
		window.sessionStorage.setItem('mestier.authRecoveryAt', String(Date.now()))

		const { removeUser, signinRedirect } = mountGate({ error: REFRESH_FAILURE })

		await waitFor(() => {
			expect(screen.getByText("Erreur d'authentification")).toBeDefined()
		})
		expect(removeUser).not.toHaveBeenCalled()
		expect(signinRedirect).not.toHaveBeenCalled()
	})

	it('lets the user take over from the error screen', async () => {
		const { removeUser, signinRedirect } = mountGate({
			error: errorFrom('signinCallback', 'Sign-in failed'),
		})

		await userEvent.click(
			screen.getByRole('button', { name: 'Se reconnecter' }),
		)

		await waitFor(() => {
			expect(removeUser).toHaveBeenCalledTimes(1)
		})
		await waitFor(() => {
			expect(signinRedirect).toHaveBeenCalledTimes(1)
		})
	})

	it('renders the workspace once authenticated', () => {
		mountGate({ isAuthenticated: true })

		expect(screen.getByText('Espace de travail')).toBeDefined()
	})
})
