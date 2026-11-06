import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCollections, useDeleteCollection } from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import { ImportCollectionFeature } from '#/pages/automation/feature/import-collection-feature'
import { collectionPath } from '#/pages/automation/lib/automation-paths'
import {
	collectionErrorMessage,
	collectionLoadErrorMessage,
} from '#/pages/automation/lib/collection-format'
import { CollectionsList } from '#/pages/automation/ui/collections-list'

export function CollectionsListFeature() {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()
	const collections = useCollections(activeOrganization.id)
	const removeCollection = useDeleteCollection(activeOrganization.id)
	const [importing, setImporting] = useState(false)

	return (
		<>
			<CollectionsList
				organizationSlug={activeOrganization.slug}
				collections={collections.data ?? []}
				isLoading={collections.isPending}
				error={
					collectionLoadErrorMessage(collections.error) ??
					collectionErrorMessage(removeCollection.error)
				}
				onImport={() => setImporting(true)}
				onRemove={(collection) => removeCollection.mutate(collection.id)}
			/>
			{importing && (
				<ImportCollectionFeature
					onCancel={() => setImporting(false)}
					onImported={(collection) => {
						setImporting(false)
						void navigate({
							to: buildOrgPath(
								activeOrganization.slug,
								collectionPath(collection.id),
							),
						})
					}}
				/>
			)}
		</>
	)
}
