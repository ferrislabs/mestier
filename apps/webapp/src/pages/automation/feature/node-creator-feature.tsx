import { Link } from '@tanstack/react-router'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import { useCollections } from '#/pages/automation/feature/use-collections'
import type { ConnectorPreset } from '#/pages/automation/feature/use-workflow-graph'
import {
	allCategories,
	COLLECTIONS_CATEGORY_ID,
	COLLECTIONS_META,
	type CreatorCategory,
	type CreatorKind,
	creatorCategories,
	searchCreatorKinds,
	TRIGGERS_CATEGORY_ID,
	triggersCategory,
} from '#/pages/automation/lib/creator'
import {
	collectionOperations,
	operationPreset,
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

function httpDescriptor(
	descriptors: readonly Schemas.ConnectorDescriptorResponse[],
) {
	return descriptors.find(
		(descriptor) =>
			descriptor.family === 'http' &&
			descriptor.fields.some((field) => field.name === 'method') &&
			descriptor.fields.some((field) => field.name === 'url'),
	)
}

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
	const collections = useCollections().data
	const startsOnTriggers = !hasTrigger
	const [query, setQuery] = useState('')
	const [categoryId, setCategoryId] = useState<string | null>(
		startsOnTriggers ? TRIGGERS_CATEGORY_ID : null,
	)
	const [collectionId, setCollectionId] = useState<string | null>(null)

	const http = httpDescriptor(descriptors)
	const connectorCategories = creatorCategories(descriptors)
	const collectionsCategory: CreatorCategory | null = http
		? {
				id: COLLECTIONS_CATEGORY_ID,
				...COLLECTIONS_META,
				kinds: [],
			}
		: null
	const categories = collectionsCategory
		? [
				...connectorCategories.slice(0, 1),
				collectionsCategory,
				...connectorCategories.slice(1),
			]
		: connectorCategories
	const category =
		categoryId === TRIGGERS_CATEGORY_ID
			? triggersCategory()
			: (categories.find((item) => item.id === categoryId) ?? null)
	const collection = collections.find((item) => item.id === collectionId)
	const atTriggerRoot = startsOnTriggers && categoryId === TRIGGERS_CATEGORY_ID

	const view: NodeCreatorView = query.trim()
		? {
				type: 'results',
				groups: searchCreatorKinds(allCategories(descriptors), query),
				operations: http ? searchOperations(collections, query) : [],
			}
		: collection
			? { type: 'operations', operations: collectionOperations(collection) }
			: categoryId === COLLECTIONS_CATEGORY_ID
				? {
						type: 'collections',
						collections: collections.map((item) => ({
							id: item.id,
							title: item.title,
							operationCount: item.operations.length,
						})),
					}
				: category
					? { type: 'kinds', kinds: category.kinds }
					: {
							type: 'categories',
							categories,
							triggerEntry: triggersCategory(),
						}

	const title = atTriggerRoot
		? 'Qu’est-ce qui déclenche ce workflow ?'
		: (collection?.title ??
			category?.label ??
			'Que doit-il se passer ensuite ?')

	const description = atTriggerRoot
		? 'Un déclencheur démarre le workflow. Vous pourrez en ajouter d’autres.'
		: collection
			? collection.baseUrl
			: category?.description

	const goBack = collection
		? () => setCollectionId(null)
		: category && !atTriggerRoot
			? () => setCategoryId(null)
			: undefined

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
			query={query}
			onQueryChange={setQuery}
			view={view}
			onPickCategory={(picked) => {
				setQuery('')
				setCategoryId(picked.id)
			}}
			onPickKind={pickKind}
			onPickCollection={(id) => {
				setQuery('')
				setCollectionId(id)
			}}
			onPickOperation={(entry) => {
				const owner = collections.find((item) => item.id === entry.collectionId)
				if (owner && http) {
					onCreate({
						type: 'connector',
						descriptor: http,
						preset: operationPreset(owner, entry.operation),
					})
				}
			}}
			emptyCollectionsAction={
				<Button asChild variant="outline" size="sm">
					<Link
						to={buildOrgPath(organizationSlug, '/automatisation/collections')}
					>
						Importer une collection
					</Link>
				</Button>
			}
			onClose={onClose}
		/>
	)
}
