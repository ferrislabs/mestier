import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	UpdateCollectionFeature,
	type UpdateOutcome,
} from '#/pages/automation/feature/update-collection-feature'
import { useCollections } from '#/pages/automation/feature/use-collections'
import { CollectionDetail } from '#/pages/automation/ui/collection-detail'

export function CollectionDetailFeature({
	collectionId,
}: {
	collectionId: string
}) {
	const { activeOrganization } = useActiveOrganization()
	const collection = useCollections().data.find(
		(item) => item.id === collectionId,
	)
	const [updating, setUpdating] = useState<{ autoCheck: boolean } | null>(null)
	const [outcome, setOutcome] = useState<UpdateOutcome | null>(null)

	return (
		<>
			<CollectionDetail
				organizationSlug={activeOrganization.slug}
				collection={collection}
				outcome={outcome}
				onUpdate={(autoCheck) => setUpdating({ autoCheck })}
			/>
			{collection && updating && (
				<UpdateCollectionFeature
					collection={collection}
					autoCheck={updating.autoCheck}
					onClose={() => setUpdating(null)}
					onUpdated={(result) => {
						setUpdating(null)
						setOutcome(result)
					}}
				/>
			)}
		</>
	)
}
