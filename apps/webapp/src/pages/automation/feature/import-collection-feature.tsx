import { useState } from 'react'
import {
	loadErrorMessage,
	loadSpecFromFile,
	loadSpecFromUrl,
} from '#/pages/automation/feature/spec-loader'
import { useAddCollection } from '#/pages/automation/feature/use-collections'
import type { Collection } from '#/pages/automation/lib/collections'
import { ImportCollectionDialog } from '#/pages/automation/ui/import-collection-dialog'

export function ImportCollectionFeature({
	onCancel,
	onImported,
}: {
	onCancel: () => void
	onImported: (collection: Collection) => void
}) {
	const addCollection = useAddCollection()
	const [loading, setLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)
	const [preview, setPreview] = useState<Collection | null>(null)

	const load = async (loader: () => Promise<Collection>) => {
		setLoading(true)
		setError(null)
		try {
			setPreview(await loader())
		} catch (caught) {
			setError(loadErrorMessage(caught))
		} finally {
			setLoading(false)
		}
	}

	return (
		<ImportCollectionDialog
			loading={loading}
			error={error}
			preview={preview}
			onLoadUrl={(url) => void load(() => loadSpecFromUrl(url))}
			onLoadFile={(file) => void load(() => loadSpecFromFile(file))}
			onSourceChange={() => setError(null)}
			onBack={() => setPreview(null)}
			onCancel={onCancel}
			onConfirm={() => {
				if (preview) void addCollection.mutateAsync(preview).then(onImported)
			}}
		/>
	)
}
