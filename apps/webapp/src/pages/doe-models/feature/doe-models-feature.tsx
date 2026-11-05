import { DoeModelsPageFeature } from '#/pages/doe-models/feature/doe-models-page-feature'

interface DoeModelsFeatureProps {
	includeArchived: boolean
	onIncludeArchivedChange: (includeArchived: boolean) => void
}

export function DoeModelsFeature({
	includeArchived,
	onIncludeArchivedChange,
}: DoeModelsFeatureProps) {
	return (
		<DoeModelsPageFeature
			embedded
			status={includeArchived ? 'archives' : 'actifs'}
			onStatusChange={(status) =>
				onIncludeArchivedChange(status === 'archives')
			}
		/>
	)
}
