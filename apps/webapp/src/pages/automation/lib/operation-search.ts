import type {
	Collection,
	CollectionOperation,
} from '#/pages/automation/lib/collections'
import type { NodeOperationRef } from '#/pages/automation/lib/editor-nodes'
import { exampleOf } from '#/pages/automation/lib/schema-typescript'
import { normalizeText } from '#/pages/automation/lib/text'

export interface OperationMatch {
	collectionId: string
	collectionTitle: string
	operation: CollectionOperation
}

export function collectionOperations(collection: Collection): OperationMatch[] {
	return collection.operations.map((operation) => ({
		collectionId: collection.id,
		collectionTitle: collection.title,
		operation,
	}))
}

export function searchOperations(
	collections: readonly Collection[],
	query: string,
): OperationMatch[] {
	const needle = normalizeText(query)
	if (needle === '') return []
	return collections
		.flatMap(collectionOperations)
		.filter((match) =>
			normalizeText(
				`${match.operation.summary} ${match.operation.path} ${match.operation.tag} ${match.collectionTitle}`,
			).includes(needle),
		)
}

export function operationPreset(
	collection: Collection,
	operation: CollectionOperation,
): {
	config: Record<string, unknown>
	meta: { label: string; subtitle: string; operation: NodeOperationRef }
} {
	return {
		config: {
			method: operation.method,
			url: `${collection.baseUrl.replace(/\/$/, '')}${operation.path}`,
		},
		meta: {
			label: operation.summary,
			subtitle: `${operation.method} ${operation.path}`,
			operation: { collectionId: collection.id, operationId: operation.id },
		},
	}
}

export interface ResolvedOperation {
	collection: Collection
	operation: CollectionOperation
}

export function resolveOperation(
	ref: NodeOperationRef,
	collections: readonly Collection[],
): ResolvedOperation | null {
	const collection = collections.find((item) => item.id === ref.collectionId)
	const operation = collection?.operations.find(
		(item) => item.id === ref.operationId,
	)
	return collection && operation ? { collection, operation } : null
}

const SUCCESS_STATUSES = new Set(['200', '201'])

export function operationSuccessExample({
	collection,
	operation,
}: ResolvedOperation): {
	summary: string
	status: string
	example: unknown
} | null {
	if (
		!operation.response ||
		!operation.responseStatus ||
		!SUCCESS_STATUSES.has(operation.responseStatus)
	) {
		return null
	}
	return {
		summary: operation.summary,
		status: operation.responseStatus,
		example: exampleOf(operation.response, collection.schemas),
	}
}
