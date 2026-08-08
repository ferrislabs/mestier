import { AlertCircle, Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { OrganizationListProvider } from '#/hooks/use-active-organization'
import { useMyOrganizations } from '#/hooks/use-organizations'
import { OnboardingFeature } from '#/pages/onboarding/feature/onboarding-feature'

interface OrgGateProps {
	children: React.ReactNode
}

export function OrgGate({ children }: OrgGateProps) {
	const { data, isLoading, isError, refetch } = useMyOrganizations()

	if (isLoading) {
		return (
			<FullscreenMessage
				icon={<Loader2 className="size-8 animate-spin text-primary" />}
				title="Chargement…"
				message="Récupération de vos organisations"
			/>
		)
	}

	if (isError) {
		return (
			<FullscreenMessage
				icon={<AlertCircle className="size-8 text-destructive" />}
				title="Impossible de charger vos organisations"
				message="Une erreur réseau s'est produite. Vérifiez votre connexion et réessayez."
				action={<Button onClick={() => void refetch()}>Réessayer</Button>}
			/>
		)
	}

	if (!data?.data || data.data.length === 0) {
		return <OnboardingFeature />
	}

	return (
		<OrganizationListProvider organizations={data.data}>
			{children}
		</OrganizationListProvider>
	)
}

export { FullscreenMessage }

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
