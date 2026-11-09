import { useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { EmptyState } from '#/components/empty-state'
import { PageShell } from '#/components/ui/surface'
import { MyTriadsFeature } from '#/pages/cursus/me/feature/my-triads-feature'
import { usePortalSpace } from '#/pages/cursus/portal/feature/use-portal-space'

interface PortalSpaceFeatureProps {
	organizationSlug: string
}

export function PortalSpaceFeature({
	organizationSlug,
}: PortalSpaceFeatureProps) {
	const navigate = useNavigate()
	const { space, isLoading } = usePortalSpace(organizationSlug)
	const isMember = space?.is_member === true

	useEffect(() => {
		if (isMember) {
			void navigate({
				to: '/o/$organizationSlug/cursus/mes-trinomes',
				params: { organizationSlug },
				replace: true,
			})
		}
	}, [isMember, navigate, organizationSlug])

	if (isLoading || isMember) {
		return (
			<PageShell>
				<p className="text-muted-foreground">Chargement…</p>
			</PageShell>
		)
	}
	if (!space) {
		return (
			<PageShell>
				<EmptyState
					title="Espace introuvable"
					hint="Vous n’avez pas d’accès Cursus dans cette organisation."
				/>
			</PageShell>
		)
	}

	return (
		<MyTriadsFeature
			organizationId={space.organization_id}
			title={space.name}
			onOpen={(triadId) =>
				void navigate({
					to: '/portail/$organizationSlug/trinomes/$triadId',
					params: { organizationSlug, triadId },
				})
			}
		/>
	)
}
