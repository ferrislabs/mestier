import type { ReactNode } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useRhModel } from '#/pages/hr/feature/use-rh-model'
import { RhProvider } from '#/pages/hr/ui/rh/shared/rh-context'
import { RhLoadError, RhLoading } from '#/pages/hr/ui/rh/shared/rh-loading'

interface RhScreenProps {
	loadingLabel: string
	children: ReactNode
}

export function RhScreen({ loadingLabel, children }: RhScreenProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<RhScreenBody
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			loadingLabel={loadingLabel}
		>
			{children}
		</RhScreenBody>
	)
}

interface RhScreenBodyProps extends RhScreenProps {
	organizationId: string
	organizationSlug: string
}

function RhScreenBody({
	organizationId,
	organizationSlug,
	loadingLabel,
	children,
}: RhScreenBodyProps) {
	const model = useRhModel(organizationId, organizationSlug)

	if (model.error) return <RhLoadError message={model.error} />
	if (model.isLoading) return <RhLoading label={loadingLabel} />

	return <RhProvider value={model.context}>{children}</RhProvider>
}
