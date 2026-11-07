import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCollection } from '#/hooks/use-automation'
import {
	UpdateCollectionFeature,
	type UpdateOutcome,
} from '#/pages/automation/feature/update-collection-feature'
import {
	collectionLoadErrorMessage,
	isCollectionMissing,
} from '#/pages/automation/lib/collection-format'
import { CollectionDetail } from '#/pages/automation/ui/collection-detail'

export function CollectionDetailFeature({
	collectionId,
}: {
	collectionId: string
}) {
	const { activeOrganization } = useActiveOrganization()
	const collection = useCollection(activeOrganization.id, collectionId)
	const [updating, setUpdating] = useState<{ autoCheck: boolean } | null>(null)
	const [outcome, setOutcome] = useState<UpdateOutcome | null>(null)

	return (
		<>
			<CollectionDetail
				organizationSlug={activeOrganization.slug}
				collection={collection.data}
				isLoading={collection.isPending}
				error={
					isCollectionMissing(collection.error)
						? null
						: collectionLoadErrorMessage(collection.error)
				}
				outcome={outcome}
				onUpdate={(autoCheck) => setUpdating({ autoCheck })}
			/>
			{collection.data && updating && (
				<UpdateCollectionFeature
					collection={collection.data}
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
