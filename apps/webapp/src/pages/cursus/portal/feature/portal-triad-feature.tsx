import { useNavigate } from '@tanstack/react-router'
import { PageShell } from '#/components/ui/surface'
import { MyTriadFeature } from '#/pages/cursus/me/feature/my-triad-feature'
import { TriadNotFound } from '#/pages/cursus/me/ui/triad-not-found'
import { usePortalSpace } from '#/pages/cursus/portal/feature/use-portal-space'

interface PortalTriadFeatureProps {
	organizationSlug: string
	triadId: string
}

export function PortalTriadFeature({
	organizationSlug,
	triadId,
}: PortalTriadFeatureProps) {
	const navigate = useNavigate()
	const { space, isLoading } = usePortalSpace(organizationSlug)
	const back = () =>
		void navigate({
			to: '/portail/$organizationSlug',
			params: { organizationSlug },
		})

	if (!isLoading && !space) return <TriadNotFound onBack={back} />
	if (!space) {
		return (
			<PageShell>
				<p className="text-muted-foreground">Chargement…</p>
			</PageShell>
		)
	}

	return (
		<MyTriadFeature
			organizationId={space.organization_id}
			triadId={triadId}
			onBack={back}
		/>
	)
}
