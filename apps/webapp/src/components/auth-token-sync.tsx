import { useEffect } from 'react'
import { useAuth } from 'react-oidc-context'

import { clearAuth, setAuth } from '#/store/auth.store'

export function AuthTokenSync() {
	const auth = useAuth()

	useEffect(() => {
		if (auth.isAuthenticated && auth.user?.access_token) {
			setAuth({
				accessToken: auth.user.access_token,
				expiresAt: auth.user.expires_at ?? null,
				isAuthenticated: true,
			})
		} else if (!auth.isLoading) {
			clearAuth()
		}
	}, [
		auth.isAuthenticated,
		auth.isLoading,
		auth.user?.access_token,
		auth.user?.expires_at,
	])

	return null
}
