import type { LucideIcon } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import {
	CATEGORY_IDS,
	type CategoryId,
	categoryCopy,
	describeConnector,
	TRIGGER_COPY_KINDS,
	type TriggerCopyKind,
	triggerCopy,
} from '#/pages/automation/lib/connector-copy'
import { defaultSchedule } from '#/pages/automation/lib/schedule'
import { normalizeText } from '#/pages/automation/lib/text'
import { defaultWebhook } from '#/pages/automation/lib/webhook-contract'

export type CreatorSource =
	| { type: 'connector'; descriptor: Schemas.ConnectorDescriptorResponse }
	| { type: 'trigger'; kind: Schemas.TriggerKindDto }

export interface CreatorKind {
	id: string
	label: string
	description: string
	icon: LucideIcon
	hue: number
	group: string | null
	source: CreatorSource
}

export interface CreatorKindGroup {
	label: string | null
	kinds: CreatorKind[]
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
			group: descriptor.group ?? null,
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

const NEW_TRIGGER_KINDS: Record<TriggerCopyKind, () => Schemas.TriggerKindDto> =
	{
		schedule: () => ({ Schedule: defaultSchedule() }),
		events: () => ({ Events: [] }),
		webhook: () => ({ Webhook: defaultWebhook() }),
		manual: () => 'Manual',
	}

function triggerKinds(): CreatorKind[] {
	return TRIGGER_COPY_KINDS.map((kind) => {
		const copy = triggerCopy(kind)
		return {
			id: `trigger:${kind}`,
			label: copy.label,
			description: copy.description,
			icon: copy.icon,
			hue: copy.hue,
			group: null,
			source: { type: 'trigger', kind: NEW_TRIGGER_KINDS[kind]() },
		}
	})
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

export function groupCreatorKinds(
	kinds: readonly CreatorKind[],
): CreatorKindGroup[] {
	const byGroup = new Map<string | null, CreatorKind[]>()
	for (const kind of kinds) {
		const bucket = byGroup.get(kind.group)
		if (bucket) bucket.push(kind)
		else byGroup.set(kind.group, [kind])
	}
	return [...byGroup]
		.map(([label, grouped]) => ({
			label,
			kinds: grouped.sort(
				(a, b) =>
					a.label.localeCompare(b.label, 'fr') || a.id.localeCompare(b.id),
			),
		}))
		.sort((a, b) =>
			a.label === null
				? -1
				: b.label === null
					? 1
					: a.label.localeCompare(b.label, 'fr'),
		)
}

function searchHaystack(kind: CreatorKind): string {
	return normalizeText(`${kind.label} ${kind.description} ${kind.group ?? ''}`)
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
				(kind) => needle === '' || searchHaystack(kind).includes(needle),
			),
		}))
		.filter((category) => category.kinds.length > 0)
}
