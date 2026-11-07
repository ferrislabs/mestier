import { CalendarClock, Library, Webhook } from 'lucide-react'
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
]

describe('connectorCopy', () => {
	it('knows every kind of the server catalogue', () => {
		for (const kind of SERVER_KINDS) {
			expect(connectorCopy(kind), kind).toBeDefined()
		}
	})

	it('gives each kind a distinct hue and a non-empty French copy', () => {
		const hues = SERVER_KINDS.map((kind) => connectorCopy(kind)?.hue)
		expect(new Set(hues).size).toBe(SERVER_KINDS.length)
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
