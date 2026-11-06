import { useEffect, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useReplaceCollection } from '#/hooks/use-automation'
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
	const { activeOrganization } = useActiveOrganization()
	const replaceCollection = useReplaceCollection(activeOrganization.id)
	const [installed] = useState(collection)
	const sourceUrl = installed.source.kind === 'url' ? installed.source.url : ''
	const [loading, setLoading] = useState(autoCheck && sourceUrl !== '')
	const [loadError, setLoadError] = useState<string | null>(null)
	const [candidate, setCandidate] = useState<UpdateCandidate | null>(null)

	const compare = (draft: CollectionDraft): UpdateCandidate => ({
		draft,
		diff: diffCollections(installed, draft),
	})

	const load = async (loader: () => Promise<CollectionDraft>) => {
		setLoading(true)
		setLoadError(null)
		try {
			setCandidate(compare(await loader()))
		} catch (caught) {
			setLoadError(loadErrorMessage(caught))
		} finally {
			setLoading(false)
		}
	}

	useEffect(() => {
		if (!autoCheck || !sourceUrl) return
		let cancelled = false
		loadSpecFromUrl(sourceUrl)
			.then((draft) => {
				if (!cancelled)
					setCandidate({
						draft,
						diff: diffCollections(installed, draft),
					})
			})
			.catch((caught: unknown) => {
				if (!cancelled) setLoadError(loadErrorMessage(caught))
			})
			.finally(() => {
				if (!cancelled) setLoading(false)
			})
		return () => {
			cancelled = true
		}
	}, [autoCheck, sourceUrl, installed])

	return (
		<UpdateCollectionDialog
			collection={installed}
			sourceUrl={sourceUrl}
			loading={loading}
			saving={replaceCollection.isPending}
			error={loadError ?? collectionErrorMessage(replaceCollection.error)}
			candidate={candidate}
			onLoadUrl={(url) => void load(() => loadSpecFromUrl(url))}
			onLoadFile={(file) => void load(() => loadSpecFromFile(file))}
			onSourceChange={() => {
				setLoadError(null)
				replaceCollection.reset()
			}}
			onResetCandidate={() => {
				setCandidate(null)
				replaceCollection.reset()
			}}
			onClose={onClose}
			onConfirm={() => {
				if (!candidate) return
				replaceCollection.mutate(
					{
						collectionId: installed.id,
						draft: withStableOperationIds(installed, candidate.draft),
					},
					{
						onSuccess: () =>
							onUpdated({
								version: candidate.draft.version,
								breakingCount: candidate.diff.breakingCount,
							}),
					},
				)
			}}
		/>
	)
}
