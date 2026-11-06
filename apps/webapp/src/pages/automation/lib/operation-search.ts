import type {
	Collection,
	CollectionOperation,
} from '#/pages/automation/lib/collections'
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
