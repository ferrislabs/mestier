import {
	Activity,
	BadgeCheck,
	Boxes,
	CalendarClock,
	CalendarSync,
	Contact,
	GitBranch,
	Globe,
	Library,
	type LucideIcon,
	MousePointerClick,
	ReceiptText,
	Repeat,
	SlidersHorizontal,
	Split,
	UserCheck,
	UserCog,
	UserPlus,
	UserRoundPlus,
	Webhook,
	Wrench,
	Zap,
} from 'lucide-react'

export type CategoryId =
	| 'triggers'
	| 'actions'
	| 'collections'
	| 'flow'
	| 'core'
	| 'human'

export interface CategoryCopy {
	id: CategoryId
	label: string
	description: string
	icon: LucideIcon
}

export interface ConnectorCopy {
	label: string
	subtitle: string
	description: string
	icon: LucideIcon
	hue: number
	category: CategoryId
	order: number
	internal: boolean
}

export interface TriggerCopy {
	label: string
	subtitle: string
	description: string
	icon: LucideIcon
	hue: number
}

export type TriggerCopyKind = 'schedule' | 'events' | 'webhook' | 'manual'

export const TRIGGER_COPY_KINDS: readonly TriggerCopyKind[] = [
	'schedule',
	'events',
	'webhook',
	'manual',
]

export const CATEGORY_IDS: readonly CategoryId[] = [
	'actions',
	'collections',
	'flow',
	'core',
	'human',
]

const CATEGORIES: Record<CategoryId, CategoryCopy> = {
	triggers: {
		id: 'triggers',
		label: 'Déclencheurs',
		description:
			'Ce qui démarre le workflow. Un workflow peut en avoir plusieurs.',
		icon: Zap,
	},
	actions: {
		id: 'actions',
		label: 'Actions Mestier',
		description: 'Agir dans Mestier : projets, tickets, devis, discussions',
		icon: Boxes,
	},
	collections: {
		id: 'collections',
		label: 'Collections',
		description: 'Appeler une API importée depuis une spec OpenAPI',
		icon: Library,
	},
	flow: {
		id: 'flow',
		label: 'Logique',
		description: 'Brancher, fusionner ou mettre en pause le workflow',
		icon: Split,
	},
	core: {
		id: 'core',
		label: 'Outils',
		description: 'Exécuter du code, envoyer un e-mail, appeler une API',
		icon: Wrench,
	},
	human: {
		id: 'human',
		label: 'Validation humaine',
		description: 'Demander l’accord d’un membre de l’équipe avant de continuer',
		icon: UserCheck,
	},
}

const TRIGGERS: Record<TriggerCopyKind, TriggerCopy> = {
	schedule: {
		label: 'Planificateur',
		subtitle: 'Déclencheur planifié',
		description: 'Se déclenche à intervalle régulier',
		icon: CalendarClock,
		hue: 255,
	},
	webhook: {
		label: 'Webhook',
		subtitle: 'Déclencheur HTTP',
		description: 'Se déclenche quand un service externe appelle une URL',
		icon: Webhook,
		hue: 335,
	},
	manual: {
		label: 'Clic',
		subtitle: 'Déclenchement manuel',
		description: 'Se déclenche quand vous cliquez sur « Tester le workflow »',
		icon: MousePointerClick,
		hue: 220,
	},
	events: {
		label: 'Event Mestier',
		subtitle: 'Déclencheur Mestier',
		description: 'Se déclenche quand un événement se produit dans Mestier',
		icon: Activity,
		hue: 165,
	},
}

const CONNECTORS: Record<string, ConnectorCopy> = {
	'mestier.customer.create': {
		label: 'Créer un client',
		subtitle: 'Action Mestier',
		description: 'Crée un client dans Mestier',
		icon: UserRoundPlus,
		hue: 120,
		category: 'actions',
		order: 1,
		internal: false,
	},
	'mestier.support.assign_ticket': {
		label: 'Assigner un ticket',
		subtitle: 'Action Mestier',
		description: 'Assigne un ticket à un membre de l’équipe',
		icon: UserPlus,
		hue: 30,
		category: 'actions',
		order: 2,
		internal: false,
	},
	'mestier.support.add_badge': {
		label: 'Ajouter un badge au ticket',
		subtitle: 'Action Mestier',
		description: 'Ajoute un badge de couleur à un ticket',
		icon: BadgeCheck,
		hue: 75,
		category: 'actions',
		order: 4,
		internal: false,
	},
	'mestier.task_recurrence.extend_horizon': {
		label: 'Étendre l’horizon de récurrence',
		subtitle: 'Action Mestier',
		description: 'Prolonge les tâches récurrentes sur l’horizon à venir',
		icon: CalendarSync,
		hue: 195,
		category: 'actions',
		order: 99,
		internal: true,
	},
	'collection.operation': {
		label: 'Opération de collection',
		subtitle: 'Collection',
		description: 'Appelle une opération d’une spec OpenAPI importée',
		icon: Library,
		hue: 200,
		category: 'collections',
		order: 1,
		internal: false,
	},
	'flow.condition': {
		label: 'Si',
		subtitle: 'Condition',
		description: 'Sépare le flux selon une condition',
		icon: GitBranch,
		hue: 145,
		category: 'flow',
		order: 1,
		internal: false,
	},
	'flow.loop': {
		label: 'Pour chaque',
		subtitle: 'Boucle',
		description: 'Répète les étapes suivantes pour chaque élément d’une liste',
		icon: Repeat,
		hue: 175,
		category: 'flow',
		order: 2,
		internal: false,
	},
	'flow.config': {
		label: 'Configuration',
		subtitle: 'Variables',
		description:
			'Définit des variables réutilisables dans la suite du workflow',
		icon: SlidersHorizontal,
		hue: 55,
		category: 'flow',
		order: 4,
		internal: false,
	},
	'http.request': {
		label: 'Requête HTTP',
		subtitle: 'Requête HTTP',
		description: 'Appelle une API externe',
		icon: Globe,
		hue: 265,
		category: 'core',
		order: 3,
		internal: false,
	},
	'odoo.create_partner': {
		label: 'Créer un partenaire Odoo',
		subtitle: 'Odoo · Partenaire',
		description: 'Crée un contact dans Odoo',
		icon: Contact,
		hue: 310,
		category: 'core',
		order: 4,
		internal: false,
	},
	'odoo.update_partner': {
		label: 'Mettre à jour un partenaire Odoo',
		subtitle: 'Odoo · Partenaire',
		description: 'Met à jour un contact existant dans Odoo',
		icon: UserCog,
		hue: 320,
		category: 'core',
		order: 5,
		internal: false,
	},
	'odoo.create_invoice': {
		label: 'Créer une facture Odoo',
		subtitle: 'Odoo · Facture',
		description: 'Crée une facture client dans Odoo',
		icon: ReceiptText,
		hue: 330,
		category: 'core',
		order: 6,
		internal: false,
	},
}

const FALLBACK_CATEGORY_BY_FAMILY: Record<string, CategoryId> = {
	mestier: 'actions',
	flow: 'flow',
}

const FALLBACK_SUBTITLE = 'Connecteur'
const FALLBACK_ICON = Wrench
const FALLBACK_HUE = 220
const FALLBACK_ORDER = 50

export function categoryCopy(id: CategoryId): CategoryCopy {
	return CATEGORIES[id]
}

export function triggerCopy(kind: TriggerCopyKind): TriggerCopy {
	return TRIGGERS[kind]
}

export function connectorCopy(kind: string): ConnectorCopy | undefined {
	return CONNECTORS[kind]
}

export function isInternalConnector(kind: string): boolean {
	return CONNECTORS[kind]?.internal ?? false
}

export function describeConnector(descriptor: {
	kind: string
	family: string
	label: string
}): ConnectorCopy {
	return (
		CONNECTORS[descriptor.kind] ?? {
			label: descriptor.label,
			subtitle: FALLBACK_SUBTITLE,
			description: '',
			icon: FALLBACK_ICON,
			hue: FALLBACK_HUE,
			category: FALLBACK_CATEGORY_BY_FAMILY[descriptor.family] ?? 'core',
			order: FALLBACK_ORDER,
			internal: false,
		}
	)
}
