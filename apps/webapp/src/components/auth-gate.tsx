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
 * Sources d'erreur qui signifient « la session stockée ne peut plus être
 * prolongée » : jeton de rafraîchissement expiré, révoqué, ou consommé par un
 * autre onglet. Ce n'est pas une panne, c'est une fin de session — on repart
 * en connexion au lieu d'afficher une erreur.
 */
const SESSION_ENDED_SOURCES: ReadonlySet<ErrorContext['source']> = new Set([
	'renewSilent',
	'signinSilent',
])

/**
 * Marqueur anti-boucle. Une reconnexion échouée qui repartirait aussitôt en
 * reconnexion enverrait l'utilisateur dans un aller-retour sans fin avec l'IdP,
 * bien pire que l'écran d'erreur qu'on cherche à éviter. Le marqueur survit au
 * rechargement de page, contrairement à un état React.
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
					// Purger l'utilisateur périmé d'abord : sans ça il reste en
					// `localStorage` et le prochain chargement rejoue le même jeton mort.
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
