import type { LucideIcon } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import {
	CATEGORY_IDS,
	type CategoryId,
	categoryCopy,
	describeConnector,
	triggerCopy,
} from '#/pages/automation/lib/connector-copy'
import { normalizeText } from '#/pages/automation/lib/text'

export type CreatorSource =
	| { type: 'connector'; descriptor: Schemas.ConnectorDescriptorResponse }
	| { type: 'trigger'; kind: Schemas.TriggerKindDto }

export interface CreatorKind {
	id: string
	label: string
	description: string
	icon: LucideIcon
	hue: number
	source: CreatorSource
}

export interface CreatorCategory {
	id: string
	label: string
	description: string
	icon: LucideIcon
	kinds: CreatorKind[]
}

export const TRIGGERS_CATEGORY_ID: CategoryId = 'triggers'
export const COLLECTIONS_CATEGORY_ID: CategoryId = 'collections'

interface RankedKind {
	kind: CreatorKind
	category: CategoryId
	order: number
}

function rankedKind(
	descriptor: Schemas.ConnectorDescriptorResponse,
): RankedKind | null {
	const copy = describeConnector(descriptor)
	if (copy.internal) return null
	return {
		category: copy.category,
		order: copy.order,
		kind: {
			id: `connector:${descriptor.kind}`,
			label: copy.label,
			description: copy.description,
			icon: copy.icon,
			hue: copy.hue,
			source: { type: 'connector', descriptor },
		},
	}
}

function byOrderThenLabel(a: RankedKind, b: RankedKind): number {
	return (
		a.order - b.order ||
		a.kind.label.localeCompare(b.kind.label, 'fr') ||
		a.kind.id.localeCompare(b.kind.id)
	)
}

function triggerKinds(): CreatorKind[] {
	const events = triggerCopy('events')
	const manual = triggerCopy('manual')
	return [
		{
			id: 'trigger:events',
			label: events.label,
			description: events.description,
			icon: events.icon,
			hue: events.hue,
			source: { type: 'trigger', kind: { Events: [] } },
		},
		{
			id: 'trigger:manual',
			label: manual.label,
			description: manual.description,
			icon: manual.icon,
			hue: manual.hue,
			source: { type: 'trigger', kind: 'Manual' },
		},
	]
}

export function triggersCategory(): CreatorCategory {
	const copy = categoryCopy(TRIGGERS_CATEGORY_ID)
	return {
		id: copy.id,
		label: copy.label,
		description: copy.description,
		icon: copy.icon,
		kinds: triggerKinds(),
	}
}

export function creatorCategories(
	descriptors: readonly Schemas.ConnectorDescriptorResponse[],
	options: { collections?: boolean } = {},
): CreatorCategory[] {
	const ranked = descriptors
		.map(rankedKind)
		.filter((entry): entry is RankedKind => entry !== null)
		.sort(byOrderThenLabel)
	return CATEGORY_IDS.flatMap((id) => {
		const kinds = ranked
			.filter((entry) => entry.category === id)
			.map((entry) => entry.kind)
		const shown =
			kinds.length > 0 ||
			(id === COLLECTIONS_CATEGORY_ID && options.collections)
		if (!shown) return []
		const copy = categoryCopy(id)
		return [
			{
				id: copy.id,
				label: copy.label,
				description: copy.description,
				icon: copy.icon,
				kinds,
			},
		]
	})
}

export function allCategories(
	descriptors: readonly Schemas.ConnectorDescriptorResponse[],
): CreatorCategory[] {
	return [triggersCategory(), ...creatorCategories(descriptors)]
}

export function searchCreatorKinds(
	categories: readonly CreatorCategory[],
	query: string,
): CreatorCategory[] {
	const needle = normalizeText(query)
	return categories
		.map((category) => ({
			...category,
			kinds: category.kinds.filter(
				(kind) =>
					needle === '' ||
					normalizeText(`${kind.label} ${kind.description}`).includes(needle),
			),
		}))
		.filter((category) => category.kinds.length > 0)
}
