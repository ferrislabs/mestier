import { Loader2, ShieldAlert } from 'lucide-react'
import { useEffect } from 'react'
import { useAuth } from 'react-oidc-context'
import { AuthTokenSync } from '#/components/auth-token-sync'
import { Button } from '#/components/ui/button'
import { getOidcConfiguration } from '#/lib/runtime-config'

interface AuthGateProps {
	children: React.ReactNode
}

export function AuthGate({ children }: AuthGateProps) {
	const auth = useAuth()
	const isConfigured = Boolean(getOidcConfiguration())

	useEffect(() => {
		if (!isConfigured) return
		if (
			!auth.isLoading &&
			!auth.isAuthenticated &&
			!auth.activeNavigator &&
			!auth.error
		) {
			void auth.signinRedirect()
		}
	}, [auth, isConfigured])

	if (!isConfigured) {
		return (
			<FullscreenMessage
				icon={<ShieldAlert className="size-8" />}
				title="Authentification non configurée"
				message="Définissez VITE_OIDC_AUTHORITY (dev) ou issuer_url dans /config.json (prod), ainsi que VITE_OIDC_CLIENT_ID."
			/>
		)
	}

	if (auth.error) {
		return (
			<FullscreenMessage
				icon={<ShieldAlert className="size-8 text-destructive" />}
				title="Erreur d'authentification"
				message={auth.error.message}
				action={
					<Button onClick={() => auth.signinRedirect()}>Réessayer</Button>
				}
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
			<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
				{icon}
			</div>
			<div>
				<p className="font-semibold">{title}</p>
				<p className="mt-1 text-sm text-muted-foreground">{message}</p>
			</div>
			{action}
		</div>
	)
}
