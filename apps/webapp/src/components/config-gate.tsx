import { Loader2 } from 'lucide-react'
import type { User } from 'oidc-client-ts'
import { useEffect, useState } from 'react'
import { AuthProvider } from 'react-oidc-context'
import { getUserManager } from '#/lib/oidc'
import { loadRuntimeConfig } from '#/lib/runtime-config'

interface ConfigGateProps {
	children: React.ReactNode
}

export function ConfigGate({ children }: ConfigGateProps) {
	const [ready, setReady] = useState(false)

	useEffect(() => {
		void loadRuntimeConfig().finally(() => setReady(true))
	}, [])

	if (!ready) {
		return (
			<div className="flex min-h-screen items-center justify-center">
				<Loader2 className="size-6 animate-spin text-primary" />
			</div>
		)
	}

	const userManager = getUserManager()
	if (!userManager) {
		return <>{children}</>
	}

	return (
		<AuthProvider userManager={userManager} onSigninCallback={onSigninCallback}>
			{children}
		</AuthProvider>
	)
}

/**
 * Wipes `code` and `state` from the URL once the IdP's return is consumed,
 * then restores whatever page originally triggered sign-in.
 *
 * `redirect_uri` is fixed to the app origin (`getOidcConfiguration().redirect_uri`),
 * not the page a caller was on — so without this, a sign-in started from
 * `/invite/$token` (via `AuthGate`'s `redirectState` prop) would land back on
 * `/` instead, stranding the token. `user.state` is opaque application data,
 * round-tripped by the IdP unmodified — see `AuthGate`'s `redirectState` doc
 * comment for the other half of this contract.
 */
export function onSigninCallback(user: User | undefined) {
	const returnTo = typeof user?.state === 'string' ? user.state : undefined

	if (returnTo && returnTo !== window.location.pathname) {
		window.location.assign(returnTo)
		return
	}

	window.history.replaceState({}, document.title, window.location.pathname)
}
