import { useMyCursusTriads } from '#/pages/cursus/me/feature/use-my-cursus'
import { MyTriadsList } from '#/pages/cursus/me/ui/my-triads-list'

interface MyTriadsFeatureProps {
	organizationId: string | undefined
	title?: string
	description?: string
	onOpen: (triadId: string) => void
}

export function MyTriadsFeature({
	organizationId,
	title = 'Mes trinômes',
	description = 'Les trinômes auxquels vous participez.',
	onOpen,
}: MyTriadsFeatureProps) {
	const triads = useMyCursusTriads(organizationId)

	return (
		<MyTriadsList
			title={title}
			description={description}
			triads={triads.data?.data ?? []}
			isLoading={triads.isLoading || organizationId === undefined}
			onOpen={onOpen}
		/>
	)
}
