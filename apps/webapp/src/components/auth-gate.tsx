import { Loader2, ShieldAlert } from 'lucide-react'
import { useEffect, useState } from 'react'
import { type ErrorContext, useAuth } from 'react-oidc-context'
import { AuthTokenSync } from '#/components/auth-token-sync'
import { Button } from '#/components/ui/button'
import { getOidcConfiguration } from '#/lib/runtime-config'

interface AuthGateProps {
	children: React.ReactNode
}

/**
 * Error sources meaning "the stored session can no longer be extended":
 * refresh token expired, revoked, or spent by another tab. This is not a
 * failure, it is the end of the session — we start signing in again instead of
 * showing an error.
 */
const SESSION_ENDED_SOURCES: ReadonlySet<ErrorContext['source']> = new Set([
	'renewSilent',
	'signinSilent',
])

/**
 * Anti-loop marker. A failed reconnection that immediately reconnected again
 * would send the user into an endless round trip with the IdP, far worse than
 * the error screen we are trying to avoid. Unlike React state, the marker
 * survives a page reload.
 */
const RECOVERY_MARKER_KEY = 'mestier.authRecoveryAt'
const RECOVERY_COOLDOWN_MS = 30_000

type RecoveryState = 'idle' | 'recovering' | 'exhausted'

export function AuthGate({ children }: AuthGateProps) {
	const auth = useAuth()
	const isConfigured = Boolean(getOidcConfiguration())
	const [recovery, setRecovery] = useState<RecoveryState>('idle')

	const sessionEnded =
		auth.error !== undefined && SESSION_ENDED_SOURCES.has(auth.error.source)
	const isRecovering = sessionEnded && recovery !== 'exhausted'

	useEffect(() => {
		if (!isConfigured) return
		if (auth.isLoading || auth.activeNavigator) return

		if (sessionEnded) {
			if (recovery !== 'idle') return

			if (recoveredRecently()) {
				setRecovery('exhausted')
				return
			}

			setRecovery('recovering')
			void (async () => {
				try {
					markRecovery()
					// Purge the stale user first: without it they stay in
					// `localStorage` and the next load replays the same dead token.
					await auth.removeUser()
					await auth.signinRedirect()
				} catch {
					setRecovery('exhausted')
				}
			})()
			return
		}

		if (!auth.isAuthenticated && !auth.error) {
			void auth.signinRedirect()
		}
	}, [auth, isConfigured, sessionEnded, recovery])

	if (!isConfigured) {
		return (
			<FullscreenMessage
				icon={<ShieldAlert className="size-8" />}
				title="Authentification non configurée"
				message="Définissez VITE_OIDC_AUTHORITY (dev) ou issuer_url dans /config.json (prod), ainsi que VITE_OIDC_CLIENT_ID."
			/>
		)
	}

	if (auth.error && !isRecovering) {
		return (
			<FullscreenMessage
				icon={<ShieldAlert className="size-8 text-destructive" />}
				title="Erreur d'authentification"
				message={auth.error.message}
				action={
					<Button
						onClick={() => {
							void auth.removeUser().then(() => auth.signinRedirect())
						}}
					>
						Se reconnecter
					</Button>
				}
			/>
		)
	}

	if (isRecovering) {
		return (
			<FullscreenMessage
				icon={<Loader2 className="size-8 animate-spin text-primary" />}
				title="Session expirée"
				message="Reconnexion en cours…"
			/>
		)
	}

	if (auth.isLoading || auth.activeNavigator === 'signinSilent') {
		return (
			<FullscreenMessage
				icon={<Loader2 className="size-8 animate-spin text-primary" />}
				title="Chargement…"
				message="Vérification de votre session"
			/>
		)
	}

	if (!auth.isAuthenticated) {
		return (
			<FullscreenMessage
				icon={<Loader2 className="size-8 animate-spin text-primary" />}
				title="Redirection vers le fournisseur d'identité…"
				message="Vous allez être redirigé pour vous connecter"
			/>
		)
	}

	return (
		<>
			<AuthTokenSync />
			{children}
		</>
	)
}

function recoveredRecently(): boolean {
	if (typeof window === 'undefined') return false

	const raw = window.sessionStorage.getItem(RECOVERY_MARKER_KEY)
	if (!raw) return false

	const at = Number(raw)
	if (!Number.isFinite(at)) return false

	return Date.now() - at < RECOVERY_COOLDOWN_MS
}

function markRecovery() {
	if (typeof window === 'undefined') return
	window.sessionStorage.setItem(RECOVERY_MARKER_KEY, String(Date.now()))
}

interface FullscreenMessageProps {
	icon: React.ReactNode
	title: string
	message: string
	action?: React.ReactNode
}

function FullscreenMessage({
	icon,
	title,
	message,
	action,
}: FullscreenMessageProps) {
	return (
		<div className="flex min-h-screen flex-col items-center justify-center gap-3 p-8 text-center">
			<div className="flex size-14 items-center justify-center rounded-xl border bg-card">
				{icon}
			</div>
			<div>
				<p className="font-medium">{title}</p>
				<p className="mt-1 text-sm text-muted-foreground">{message}</p>
			</div>
			{action}
		</div>
	)
}
