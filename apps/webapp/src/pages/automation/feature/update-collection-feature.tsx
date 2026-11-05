import { useEffect, useState } from 'react'
import {
	loadErrorMessage,
	loadSpecFromFile,
	loadSpecFromUrl,
} from '#/pages/automation/feature/spec-loader'
import { useUpdateCollection } from '#/pages/automation/feature/use-collections'
import type { Collection } from '#/pages/automation/lib/collections'
import {
	diffCollections,
	withStableOperationIds,
} from '#/pages/automation/lib/openapi-diff'
import {
	type UpdateCandidate,
	UpdateCollectionDialog,
} from '#/pages/automation/ui/update-collection-dialog'

export interface UpdateOutcome {
	version: string
	breakingCount: number
}

export function UpdateCollectionFeature({
	collection,
	autoCheck,
	onClose,
	onUpdated,
}: {
	collection: Collection
	autoCheck: boolean
	onClose: () => void
	onUpdated: (outcome: UpdateOutcome) => void
}) {
	const updateCollection = useUpdateCollection()
	const sourceUrl =
		collection.source.kind === 'url' ? collection.source.url : ''
	const [loading, setLoading] = useState(autoCheck && sourceUrl !== '')
	const [error, setError] = useState<string | null>(null)
	const [candidate, setCandidate] = useState<UpdateCandidate | null>(null)

	const load = async (loader: () => Promise<Collection>) => {
		setLoading(true)
		setError(null)
		try {
			const next = await loader()
			setCandidate({
				collection: next,
				diff: diffCollections(collection, next),
			})
		} catch (caught) {
			setError(loadErrorMessage(caught))
		} finally {
			setLoading(false)
		}
	}

	useEffect(() => {
		if (!autoCheck || !sourceUrl) return
		let cancelled = false
		loadSpecFromUrl(sourceUrl)
			.then((next) => {
				if (!cancelled)
					setCandidate({
						collection: next,
						diff: diffCollections(collection, next),
					})
			})
			.catch((caught: unknown) => {
				if (!cancelled) setError(loadErrorMessage(caught))
			})
			.finally(() => {
				if (!cancelled) setLoading(false)
			})
		return () => {
			cancelled = true
		}
	}, [autoCheck, sourceUrl, collection])

	return (
		<UpdateCollectionDialog
			collection={collection}
			sourceUrl={sourceUrl}
			loading={loading}
			error={error}
			candidate={candidate}
			onLoadUrl={(url) => void load(() => loadSpecFromUrl(url))}
			onLoadFile={(file) => void load(() => loadSpecFromFile(file))}
			onSourceChange={() => setError(null)}
			onResetCandidate={() => setCandidate(null)}
			onClose={onClose}
			onConfirm={() => {
				if (!candidate) return
				void updateCollection
					.mutateAsync({
						id: collection.id,
						next: withStableOperationIds(collection, candidate.collection),
					})
					.then(() =>
						onUpdated({
							version: candidate.collection.version,
							breakingCount: candidate.diff.breakingCount,
						}),
					)
			}}
		/>
	)
}
