import { Link } from '@tanstack/react-router'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCollection,
	useCollectionDetails,
	useCollections,
} from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import type { ConnectorPreset } from '#/pages/automation/feature/use-workflow-graph'
import { collectionsPath } from '#/pages/automation/lib/automation-paths'
import {
	collectionLoadErrorMessage,
	isCollectionMissing,
} from '#/pages/automation/lib/collection-format'
import {
	COLLECTION_OPERATION_KIND,
	operationPreset,
} from '#/pages/automation/lib/collection-node'
import {
	ACTIONS_CATEGORY_ID,
	allCategories,
	COLLECTIONS_CATEGORY_ID,
	type CreatorKind,
	creatorCategories,
	searchCreatorKinds,
	TRIGGERS_CATEGORY_ID,
	triggersCategory,
} from '#/pages/automation/lib/creator'
import { creatorDomains } from '#/pages/automation/lib/creator-domains'
import {
	collectionOperations,
	searchOperations,
} from '#/pages/automation/lib/operation-search'
import {
	NodeCreatorPanel,
	type NodeCreatorView,
} from '#/pages/automation/ui/node-creator-panel'

export type CreatorChoice =
	| {
			type: 'connector'
			descriptor: Schemas.ConnectorDescriptorResponse
			preset?: ConnectorPreset
	  }
	| { type: 'trigger'; kind: Schemas.TriggerKindDto }

export function NodeCreatorFeature({
	organizationSlug,
	descriptors,
	hasTrigger,
	onCreate,
	onClose,
}: {
	organizationSlug: string
	descriptors: readonly Schemas.ConnectorDescriptorResponse[]
	hasTrigger: boolean
	onCreate: (choice: CreatorChoice) => void
	onClose: () => void
}) {
	const { activeOrganization } = useActiveOrganization()
	const startsOnTriggers = !hasTrigger
	const [query, setQuery] = useState('')
	const [categoryId, setCategoryId] = useState<string | null>(
		startsOnTriggers ? TRIGGERS_CATEGORY_ID : null,
	)
	const [groupId, setGroupId] = useState<string | null>(null)
	const [collectionId, setCollectionId] = useState<string | null>(null)

	const operationDescriptor = descriptors.find(
		(descriptor) => descriptor.kind === COLLECTION_OPERATION_KIND,
	)
	const simpleDescriptors = descriptors.filter(
		(descriptor) => descriptor.kind !== COLLECTION_OPERATION_KIND,
	)
	const summaries = useCollections(activeOrganization.id)
	const summaryList = summaries.data ?? []
	const searching = query.trim() !== ''
	const selected = useCollection(activeOrganization.id, collectionId ?? '', {
		enabled: collectionId !== null,
	})
	const details = useCollectionDetails(
		activeOrganization.id,
		searching && operationDescriptor
			? summaryList.map((summary) => summary.id)
			: [],
	)
	const known = selected.data
		? [selected.data, ...details.collections]
		: details.collections

	const categories = creatorCategories(simpleDescriptors, {
		collections: operationDescriptor !== undefined,
	})
	const category =
		categoryId === TRIGGERS_CATEGORY_ID
			? triggersCategory()
			: (categories.find((item) => item.id === categoryId) ?? null)
	const domains =
		category?.id === ACTIONS_CATEGORY_ID ? creatorDomains(category.kinds) : null
	const domain = domains?.find((item) => item.id === groupId) ?? null
	const summary = summaryList.find((item) => item.id === collectionId)
	const collection = known.find((item) => item.id === collectionId)
	const atTriggerRoot = startsOnTriggers && categoryId === TRIGGERS_CATEGORY_ID

	const view: NodeCreatorView = searching
		? {
				type: 'results',
				groups: searchCreatorKinds(allCategories(simpleDescriptors), query),
				operations: operationDescriptor ? searchOperations(known, query) : [],
				loadingOperations: details.isLoading,
			}
		: collectionId
			? {
					type: 'operations',
					operations: collection ? collectionOperations(collection) : [],
					status: collection
						? 'ready'
						: selected.isError
							? isCollectionMissing(selected.error)
								? 'unavailable'
								: 'unreadable'
							: 'loading',
				}
			: categoryId === COLLECTIONS_CATEGORY_ID
				? {
						type: 'collections',
						collections: summaryList.map((item) => ({
							id: item.id,
							title: item.title,
							operationCount: item.operationCount,
						})),
						status: summaries.isPending
							? 'loading'
							: summaries.isError
								? 'failed'
								: 'ready',
						failure: collectionLoadErrorMessage(summaries.error),
					}
				: domain
					? { type: 'kinds', kinds: domain.kinds }
					: domains
						? { type: 'domains', domains }
						: category
							? { type: 'kinds', kinds: category.kinds }
							: {
									type: 'categories',
									categories,
									triggerEntry: triggersCategory(),
								}

	const title = atTriggerRoot
		? 'Qu’est-ce qui déclenche ce workflow ?'
		: collectionId
			? (collection?.title ?? summary?.title ?? 'Collection')
			: (domain?.label ?? category?.label ?? 'Que doit-il se passer ensuite ?')

	const description = atTriggerRoot
		? 'Un déclencheur démarre le workflow. Vous pourrez en ajouter d’autres.'
		: collectionId
			? (collection?.baseUrl ?? summary?.baseUrl)
			: (domain?.description ?? category?.description)

	const goBack = collectionId
		? () => setCollectionId(null)
		: domain
			? () => setGroupId(null)
			: category && !atTriggerRoot
				? () => setCategoryId(null)
				: undefined
	const backLabel = domain ? 'Revenir aux domaines' : undefined

	function pickKind(kind: CreatorKind) {
		if (kind.source.type === 'trigger') {
			onCreate({ type: 'trigger', kind: kind.source.kind })
			return
		}
		onCreate({ type: 'connector', descriptor: kind.source.descriptor })
	}

	return (
		<NodeCreatorPanel
			title={title}
			description={description}
			onBack={goBack}
			backLabel={backLabel}
			query={query}
			onQueryChange={setQuery}
			view={view}
			onPickCategory={(picked) => {
				setQuery('')
				setGroupId(null)
				setCategoryId(picked.id)
			}}
			onPickDomain={(picked) => {
				setQuery('')
				setGroupId(picked.id)
			}}
			onPickKind={pickKind}
			onPickCollection={(id) => {
				setQuery('')
				setCollectionId(id)
			}}
			onPickOperation={(entry) => {
				const owner = known.find((item) => item.id === entry.collectionId)
				if (owner && operationDescriptor) {
					onCreate({
						type: 'connector',
						descriptor: operationDescriptor,
						preset: operationPreset(owner, entry.operation),
					})
				}
			}}
			emptyCollectionsAction={
				<Button asChild variant="outline" size="sm">
					<Link to={buildOrgPath(organizationSlug, collectionsPath())}>
						Importer une collection
					</Link>
				</Button>
			}
			onClose={onClose}
		/>
	)
}
