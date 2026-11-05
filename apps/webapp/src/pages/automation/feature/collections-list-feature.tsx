import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { buildOrgPath } from '#/modules/org-path'
import { ImportCollectionFeature } from '#/pages/automation/feature/import-collection-feature'
import {
	useCollections,
	useRemoveCollection,
} from '#/pages/automation/feature/use-collections'
import { CollectionsList } from '#/pages/automation/ui/collections-list'

export function CollectionsListFeature() {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()
	const collections = useCollections()
	const removeCollection = useRemoveCollection()
	const [importing, setImporting] = useState(false)

	return (
		<>
			<CollectionsList
				organizationSlug={activeOrganization.slug}
				collections={collections.data}
				onImport={() => setImporting(true)}
				onRemove={(collection) =>
					void removeCollection.mutateAsync(collection.id)
				}
			/>
			{importing && (
				<ImportCollectionFeature
					onCancel={() => setImporting(false)}
					onImported={(collection) => {
						setImporting(false)
						void navigate({
							to: buildOrgPath(
								activeOrganization.slug,
								`/automatisation/collections/${collection.id}`,
							),
						})
					}}
				/>
			)}
		</>
	)
}
