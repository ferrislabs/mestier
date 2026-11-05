import {
	Boxes,
	Library,
	type LucideIcon,
	Split,
	Wrench,
	Zap,
} from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import {
	connectorAppearance,
	triggerAppearance,
} from '#/pages/automation/lib/connector-appearance'
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

export const TRIGGERS_CATEGORY_ID = 'triggers'
export const COLLECTIONS_CATEGORY_ID = 'collections'

interface CategoryMeta {
	label: string
	description: string
	icon: LucideIcon
}

const FAMILY_META: Record<string, CategoryMeta> = {
	mestier: {
		label: 'Actions Mestier',
		description: 'Agir dans Mestier : projets, tickets, devis, discussions',
		icon: Boxes,
	},
	flow: {
		label: 'Logique',
		description: 'Brancher, fusionner ou mettre en pause le workflow',
		icon: Split,
	},
	http: {
		label: 'Outils',
		description: 'Exécuter du code, envoyer un e-mail, appeler une API',
		icon: Wrench,
	},
}

export const COLLECTIONS_META: CategoryMeta = {
	label: 'Collections',
	description: 'Appeler une API importée depuis une spec OpenAPI',
	icon: Library,
}

const FAMILY_ORDER = ['mestier', 'flow', 'http']

function familyMeta(family: string): CategoryMeta {
	return (
		FAMILY_META[family] ?? {
			label: family.charAt(0).toUpperCase() + family.slice(1),
			description: `Connecteurs ${family}`,
			icon: Wrench,
		}
	)
}

function familyRank(family: string): number {
	const index = FAMILY_ORDER.indexOf(family)
	return index === -1 ? FAMILY_ORDER.length : index
}

function connectorKind(
	descriptor: Schemas.ConnectorDescriptorResponse,
): CreatorKind {
	const appearance = connectorAppearance(descriptor.family)
	return {
		id: `connector:${descriptor.kind}`,
		label: descriptor.label,
		description: familyMeta(descriptor.family).label,
		icon: appearance.icon,
		hue: appearance.hue,
		source: { type: 'connector', descriptor },
	}
}

function triggerKinds(): CreatorKind[] {
	const manual = triggerAppearance(true)
	const events = triggerAppearance(false)
	return [
		{
			id: 'trigger:manual',
			label: 'Clic',
			description: 'Se déclenche quand vous cliquez sur « Tester le workflow »',
			icon: manual.icon,
			hue: manual.hue,
			source: { type: 'trigger', kind: 'Manual' },
		},
		{
			id: 'trigger:events',
			label: 'Event Mestier',
			description: 'Se déclenche quand un événement se produit dans Mestier',
			icon: events.icon,
			hue: events.hue,
			source: { type: 'trigger', kind: { Events: [] } },
		},
	]
}

export function triggersCategory(): CreatorCategory {
	return {
		id: TRIGGERS_CATEGORY_ID,
		label: 'Déclencheurs',
		description:
			'Ce qui démarre le workflow. Un workflow peut en avoir plusieurs.',
		icon: Zap,
		kinds: triggerKinds(),
	}
}

export function creatorCategories(
	descriptors: readonly Schemas.ConnectorDescriptorResponse[],
): CreatorCategory[] {
	const families = [...new Set(descriptors.map((item) => item.family))].sort(
		(a, b) => familyRank(a) - familyRank(b) || a.localeCompare(b),
	)
	return families.map((family) => ({
		id: family,
		...familyMeta(family),
		kinds: descriptors
			.filter((descriptor) => descriptor.family === family)
			.map(connectorKind),
	}))
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
