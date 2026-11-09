import { useActiveOrganization } from '#/hooks/use-active-organization'
import { MyTriadFeature } from '#/pages/cursus/me/feature/my-triad-feature'
import { MyTriadsFeature } from '#/pages/cursus/me/feature/my-triads-feature'

interface ConsoleMyTriadsFeatureProps {
	onOpen: (triadId: string) => void
}

export function ConsoleMyTriadsFeature({
	onOpen,
}: ConsoleMyTriadsFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()

	return (
		<MyTriadsFeature organizationId={activeOrganizationId} onOpen={onOpen} />
	)
}

interface ConsoleMyTriadFeatureProps {
	triadId: string
	onBack: () => void
}

export function ConsoleMyTriadFeature({
	triadId,
	onBack,
}: ConsoleMyTriadFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()

	return (
		<MyTriadFeature
			organizationId={activeOrganizationId}
			triadId={triadId}
			onBack={onBack}
		/>
	)
}
