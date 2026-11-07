import {
	CalendarClock,
	FileSearch,
	FolderPlus,
	Library,
	Mail,
	Merge,
	MessageSquare,
	ReceiptText,
	StickyNote,
	Timer,
	Webhook,
	Wrench,
} from 'lucide-react'
import { describe, expect, it } from 'vitest'
import {
	CATEGORY_IDS,
	categoryCopy,
	connectorCopy,
	describeConnector,
	isInternalConnector,
	TRIGGER_COPY_KINDS,
	triggerCopy,
} from '#/pages/automation/lib/connector-copy'
import { runNodeAppearance } from '#/pages/automation/lib/run-appearance'
import { connectorTypeLabel } from '#/pages/automation/lib/run-node'
import { connectorLabel } from '#/pages/automation/lib/run-view'

const SERVER_KINDS = [
	'http.request',
	'collection.operation',
	'flow.condition',
	'flow.loop',
	'flow.config',
	'mestier.customer.create',
	'mestier.support.assign_ticket',
	'mestier.support.add_badge',
	'mestier.task_recurrence.extend_horizon',
	'odoo.create_partner',
	'odoo.update_partner',
	'odoo.create_invoice',
	'flow.wait',
	'flow.merge',
	'mestier.note.add',
	'mestier.quote.search',
	'mestier.project.create',
	'mestier.chat.post_message',
	'smtp.send',
]

const GENERATED_KINDS = [
	'mestier.invoice.send',
	'mestier.doe.generate',
	'mestier.support.close_ticket',
]

const SHARED_HUE_KINDS = ['flow.merge']

describe('connectorCopy', () => {
	it('knows every kind of the server catalogue', () => {
		for (const kind of SERVER_KINDS) {
			expect(connectorCopy(kind), kind).toBeDefined()
		}
	})

	it('gives each kind a distinct hue and a non-empty French copy', () => {
		const distinct = SERVER_KINDS.filter(
			(kind) => !SHARED_HUE_KINDS.includes(kind),
		)
		const hues = distinct.map((kind) => connectorCopy(kind)?.hue)
		expect(new Set(hues).size).toBe(distinct.length)
		for (const kind of SERVER_KINDS) {
			const copy = connectorCopy(kind)
			expect(copy?.label.length).toBeGreaterThan(0)
			expect(copy?.subtitle.length).toBeGreaterThan(0)
			expect(copy?.description.length).toBeGreaterThan(0)
		}
	})

	it('copies label, subtitle, description, icon and hue from the prototype for equivalent types', () => {
		expect(connectorCopy('mestier.support.assign_ticket')).toMatchObject({
			label: 'Assigner un ticket',
			subtitle: 'Action Mestier',
			description: 'Assigne un ticket à un membre de l’équipe',
			hue: 30,
			category: 'actions',
		})
		expect(connectorCopy('mestier.support.add_badge')).toMatchObject({
			label: 'Ajouter un badge au ticket',
			subtitle: 'Action Mestier',
			description: 'Ajoute un badge de couleur à un ticket',
			hue: 75,
			category: 'actions',
		})
		expect(connectorCopy('flow.condition')).toMatchObject({
			label: 'Si',
			subtitle: 'Condition',
			description: 'Sépare le flux selon une condition',
			hue: 145,
			category: 'flow',
		})
		expect(connectorCopy('http.request')).toMatchObject({
			label: 'Requête HTTP',
			subtitle: 'Requête HTTP',
			description: 'Appelle une API externe',
			hue: 265,
			category: 'core',
		})
	})

	it('copies the seven lot 7 nodes from the prototype', () => {
		expect(connectorCopy('flow.wait')).toMatchObject({
			label: 'Attendre',
			subtitle: 'Pause',
			description: 'Met le workflow en pause pendant une durée',
			icon: Timer,
			hue: 80,
			category: 'flow',
			internal: false,
		})
		expect(connectorCopy('flow.merge')).toMatchObject({
			label: 'Fusionner',
			subtitle: 'Fusion',
			description: 'Réunit les données de plusieurs branches',
			icon: Merge,
			hue: 200,
			category: 'flow',
			internal: false,
		})
		expect(connectorCopy('mestier.quote.search')).toMatchObject({
			label: 'Rechercher des devis',
			subtitle: 'Action Mestier',
			description: 'Recherche des devis selon leur statut et leur ancienneté',
			icon: FileSearch,
			hue: 235,
			category: 'actions',
		})
		expect(connectorCopy('mestier.project.create')).toMatchObject({
			label: 'Créer un projet',
			subtitle: 'Action Mestier',
			description: 'Crée un projet de chantier à partir d’un devis',
			icon: FolderPlus,
			hue: 150,
			category: 'actions',
		})
		expect(connectorCopy('mestier.chat.post_message')).toMatchObject({
			label: 'Publier dans un canal Discussions',
			subtitle: 'Action Mestier',
			description: 'Envoie un message dans un canal Discussions',
			icon: MessageSquare,
			hue: 300,
			category: 'actions',
		})
		expect(connectorCopy('smtp.send')).toMatchObject({
			label: 'E-mail SMTP',
			subtitle: 'Envoyer un e-mail',
			description: 'Envoie un e-mail avec votre serveur SMTP',
			icon: Mail,
			hue: 25,
			category: 'core',
		})
	})

	it('describes the note as one on a support ticket, not on a quote', () => {
		expect(connectorCopy('mestier.note.add')).toMatchObject({
			label: 'Ajouter une note',
			subtitle: 'Action Mestier',
			description: 'Ajoute une note interne sur un ticket',
			icon: StickyNote,
			hue: 85,
			category: 'actions',
		})
	})

	it('orders every category as the prototype does, the extras of the real app last', () => {
		const labelsOf = (category: string) =>
			SERVER_KINDS.filter((kind) => connectorCopy(kind)?.category === category)
				.filter((kind) => !isInternalConnector(kind))
				.sort(
					(a, b) =>
						(connectorCopy(a)?.order ?? 0) - (connectorCopy(b)?.order ?? 0),
				)
				.map((kind) => connectorCopy(kind)?.label)

		expect(labelsOf('actions')).toEqual([
			'Créer un projet',
			'Assigner un ticket',
			'Publier dans un canal Discussions',
			'Ajouter un badge au ticket',
			'Rechercher des devis',
			'Ajouter une note',
			'Créer un client',
		])
		expect(labelsOf('flow')).toEqual([
			'Si',
			'Attendre',
			'Fusionner',
			'Pour chaque',
			'Configuration',
		])
		expect(labelsOf('core')).toEqual([
			'E-mail SMTP',
			'Requête HTTP',
			'Créer un partenaire Odoo',
			'Mettre à jour un partenaire Odoo',
			'Créer une facture Odoo',
		])
	})

	it('copies the collection operation from the prototype, filed under the collections', () => {
		expect(connectorCopy('collection.operation')).toMatchObject({
			label: 'Opération de collection',
			subtitle: 'Collection',
			description: 'Appelle une opération d’une spec OpenAPI importée',
			hue: 200,
			category: 'collections',
			internal: false,
		})
		expect(connectorCopy('collection.operation')?.icon).toBe(Library)
	})

	it('keeps the server wording for the types the prototype does not have', () => {
		expect(connectorCopy('flow.loop')?.label).toBe('Pour chaque')
		expect(connectorCopy('flow.config')?.label).toBe('Configuration')
		expect(connectorCopy('mestier.customer.create')?.label).toBe(
			'Créer un client',
		)
		expect(connectorCopy('odoo.create_partner')?.label).toBe(
			'Créer un partenaire Odoo',
		)
		expect(connectorCopy('odoo.update_partner')?.label).toBe(
			'Mettre à jour un partenaire Odoo',
		)
		expect(connectorCopy('odoo.create_invoice')?.label).toBe(
			'Créer une facture Odoo',
		)
	})

	it('files the Odoo types under the tools category', () => {
		expect(connectorCopy('odoo.create_partner')?.category).toBe('core')
		expect(connectorCopy('odoo.update_partner')?.category).toBe('core')
		expect(connectorCopy('odoo.create_invoice')?.category).toBe('core')
	})

	it('flags only the horizon extension as internal', () => {
		const internal = SERVER_KINDS.filter((kind) => isInternalConnector(kind))
		expect(internal).toEqual(['mestier.task_recurrence.extend_horizon'])
	})

	it('is undefined for a kind it has never heard of', () => {
		expect(connectorCopy('nope.nothing')).toBeUndefined()
	})
})

describe('describeConnector — generated Mestier actions', () => {
	it('relies on the fallback: no hand-written entry is needed', () => {
		for (const kind of GENERATED_KINDS) {
			expect(connectorCopy(kind), kind).toBeUndefined()
		}
	})

	it('shows the server label, the group as subtitle and the server description', () => {
		const copy = describeConnector({
			kind: 'mestier.invoice.send',
			family: 'mestier',
			label: 'Envoyer une facture',
			group: 'Factures',
			description: 'Envoie une facture au client',
		})

		expect(copy).toMatchObject({
			label: 'Envoyer une facture',
			subtitle: 'Factures',
			description: 'Envoie une facture au client',
			category: 'actions',
			internal: false,
		})
		expect(copy.icon).toBe(ReceiptText)
	})

	it('leaves the description empty when the server sends none', () => {
		expect(
			describeConnector({
				kind: 'mestier.x.y',
				family: 'mestier',
				label: 'Y',
				description: null,
			}).description,
		).toBe('')
	})

	it('gives each known group its own icon and tint', () => {
		const groups = [
			'Achats',
			'Clients',
			'Devis',
			'Discussions',
			'DOE',
			'Entretien',
			'Factures',
			'Planning',
			'Rapports',
			'Référentiel',
			'Support',
			'Terrain',
		]
		const hues = groups.map(
			(group) =>
				describeConnector({
					kind: 'mestier.x.y',
					family: 'mestier',
					label: 'Y',
					group,
				}).hue,
		)

		expect(new Set(hues).size).toBe(groups.length)
	})

	it('falls back on the wrench and hue 220 for an unknown or missing group', () => {
		for (const group of ['Inconnu', null, undefined]) {
			const copy = describeConnector({
				kind: 'mestier.x.y',
				family: 'mestier',
				label: 'Y',
				group,
			})
			expect(copy.icon).toBe(Wrench)
			expect(copy.hue).toBe(220)
		}
		expect(
			describeConnector({ kind: 'mestier.x.y', family: 'mestier', label: 'Y' })
				.subtitle,
		).toBe('Connecteur')
	})
})

describe('triggerCopy', () => {
	it('copies the scheduler from the prototype', () => {
		expect(triggerCopy('schedule')).toMatchObject({
			label: 'Planificateur',
			subtitle: 'Déclencheur planifié',
			description: 'Se déclenche à intervalle régulier',
			icon: CalendarClock,
			hue: 255,
		})
	})

	it('copies the webhook from the prototype', () => {
		expect(triggerCopy('webhook')).toMatchObject({
			label: 'Webhook',
			subtitle: 'Déclencheur HTTP',
			description: 'Se déclenche quand un service externe appelle une URL',
			icon: Webhook,
			hue: 335,
		})
	})

	it('keeps the click and the Mestier event as they were', () => {
		expect(triggerCopy('manual')).toMatchObject({
			label: 'Clic',
			subtitle: 'Déclenchement manuel',
			hue: 220,
		})
		expect(triggerCopy('events')).toMatchObject({
			label: 'Event Mestier',
			subtitle: 'Déclencheur Mestier',
			hue: 165,
		})
	})

	it('lists the four triggers in the order of the prototype', () => {
		expect(TRIGGER_COPY_KINDS.map((kind) => triggerCopy(kind).label)).toEqual([
			'Planificateur',
			'Event Mestier',
			'Webhook',
			'Clic',
		])
	})
})

describe('describeConnector', () => {
	it('returns the table entry for a known kind, whatever the descriptor says', () => {
		expect(
			describeConnector({
				kind: 'flow.condition',
				family: 'flow',
				label: 'Condition',
			}),
		).toEqual(connectorCopy('flow.condition'))
	})

	it('falls back on the descriptor label and a family category for an unknown kind', () => {
		const copy = describeConnector({
			kind: 'mestier.new_thing',
			family: 'mestier',
			label: 'Nouveauté',
		})
		expect(copy.label).toBe('Nouveauté')
		expect(copy.category).toBe('actions')
		expect(copy.internal).toBe(false)
	})

	it('puts an unknown family among the tools', () => {
		expect(
			describeConnector({ kind: 'x.y', family: 'x', label: 'Y' }).category,
		).toBe('core')
	})
})

describe('triggerCopy', () => {
	it('copies the click and event triggers from the prototype', () => {
		expect(triggerCopy('manual')).toMatchObject({
			label: 'Clic',
			subtitle: 'Déclenchement manuel',
			description: 'Se déclenche quand vous cliquez sur « Tester le workflow »',
			hue: 220,
		})
		expect(triggerCopy('events')).toMatchObject({
			label: 'Event Mestier',
			description: 'Se déclenche quand un événement se produit dans Mestier',
			hue: 165,
		})
	})
})

describe('categoryCopy', () => {
	it('lists the prototype categories in its order, triggers aside', () => {
		expect(CATEGORY_IDS.map((id) => categoryCopy(id).label)).toEqual([
			'Actions Mestier',
			'Collections',
			'Logique',
			'Outils',
			'Validation humaine',
		])
	})

	it('describes each category as the prototype does', () => {
		expect(categoryCopy('triggers')).toMatchObject({
			label: 'Déclencheurs',
			description:
				'Ce qui démarre le workflow. Un workflow peut en avoir plusieurs.',
		})
		expect(categoryCopy('human').description).toBe(
			'Demander l’accord d’un membre de l’équipe avant de continuer',
		)
	})
})

describe('the execution views — a collection operation', () => {
	const operation = {
		id: 'c1',
		kind: 'collection.operation',
		version: 1,
		config: {},
	}

	it('names it after the entry until the node has a name of its own', () => {
		expect(connectorLabel(operation, undefined)).toBe('Opération de collection')
		expect(connectorTypeLabel('collection.operation', undefined)).toBe(
			'Opération de collection',
		)
		expect(
			connectorLabel({ ...operation, name: 'Lire un animal' }, undefined),
		).toBe('Lire un animal')
	})

	it('draws it with the icon and hue of the entry', () => {
		expect(
			runNodeAppearance({
				kind: 'connector',
				family: 'http',
				connector: operation,
				trigger: null,
			}),
		).toEqual({ icon: Library, hue: 200 })
	})
})
