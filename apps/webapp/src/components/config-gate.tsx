import { Loader2 } from 'lucide-react'
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

/** Wipes `code` and `state` from the URL once the IdP's return is consumed. */
function onSigninCallback() {
	window.history.replaceState({}, document.title, window.location.pathname)
}
