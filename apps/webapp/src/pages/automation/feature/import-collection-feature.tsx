import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useImportCollection } from '#/hooks/use-automation'
import {
	loadErrorMessage,
	loadSpecFromFile,
	loadSpecFromUrl,
} from '#/pages/automation/feature/spec-loader'
import { collectionErrorMessage } from '#/pages/automation/lib/collection-format'
import type {
	Collection,
	CollectionDraft,
} from '#/pages/automation/lib/collections'
import { ImportCollectionDialog } from '#/pages/automation/ui/import-collection-dialog'

export function ImportCollectionFeature({
	onCancel,
	onImported,
}: {
	onCancel: () => void
	onImported: (collection: Collection) => void
}) {
	const { activeOrganization } = useActiveOrganization()
	const importCollection = useImportCollection(activeOrganization.id)
	const [loading, setLoading] = useState(false)
	const [loadError, setLoadError] = useState<string | null>(null)
	const [preview, setPreview] = useState<CollectionDraft | null>(null)

	const load = async (loader: () => Promise<CollectionDraft>) => {
		setLoading(true)
		setLoadError(null)
		try {
			setPreview(await loader())
		} catch (caught) {
			setLoadError(loadErrorMessage(caught))
		} finally {
			setLoading(false)
		}
	}

	return (
		<ImportCollectionDialog
			loading={loading}
			saving={importCollection.isPending}
			error={loadError ?? collectionErrorMessage(importCollection.error)}
			preview={preview}
			onLoadUrl={(url) => void load(() => loadSpecFromUrl(url))}
			onLoadFile={(file) => void load(() => loadSpecFromFile(file))}
			onSourceChange={() => {
				setLoadError(null)
				importCollection.reset()
			}}
			onBack={() => {
				setPreview(null)
				importCollection.reset()
			}}
			onCancel={onCancel}
			onConfirm={() => {
				if (preview) importCollection.mutate(preview, { onSuccess: onImported })
			}}
		/>
	)
}
