import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	allCategories,
	COLLECTIONS_CATEGORY_ID,
	creatorCategories,
	searchCreatorKinds,
	TRIGGERS_CATEGORY_ID,
	triggersCategory,
} from '#/pages/automation/lib/creator'

function descriptor(
	kind: string,
	family: string,
	label: string,
): Schemas.ConnectorDescriptorResponse {
	return {
		kind,
		family,
		label,
		version: 1,
		auth: 'None',
		fields: [],
		branches: [],
		output_example: null,
	}
}

const CUSTOMER = descriptor(
	'mestier.customer.create',
	'mestier',
	'Créer un client',
)
const ASSIGN = descriptor(
	'mestier.support.assign_ticket',
	'mestier',
	'Assigner un ticket',
)
const BADGE = descriptor(
	'mestier.support.add_badge',
	'mestier',
	'Ajouter un badge',
)
const HORIZON = descriptor(
	'mestier.task_recurrence.extend_horizon',
	'mestier',
	'Étendre l’horizon de récurrence',
)
const CONDITION = descriptor('flow.condition', 'flow', 'Condition')
const LOOP = descriptor('flow.loop', 'flow', 'Pour chaque')
const CONFIG = descriptor('flow.config', 'flow', 'Configuration')
const HTTP = descriptor('http.request', 'http', 'Requête HTTP')
const ODOO_PARTNER = descriptor(
	'odoo.create_partner',
	'odoo',
	'Créer un partenaire Odoo',
)
const ODOO_INVOICE = descriptor(
	'odoo.create_invoice',
	'odoo',
	'Créer une facture Odoo',
)

const descriptors = [
	ODOO_INVOICE,
	HTTP,
	CONFIG,
	HORIZON,
	BADGE,
	LOOP,
	ODOO_PARTNER,
	CONDITION,
	ASSIGN,
	CUSTOMER,
]

describe('creatorCategories', () => {
	it('keeps the prototype order and names, and hides the categories nothing fills', () => {
		expect(creatorCategories(descriptors).map((item) => item.label)).toEqual([
			'Actions Mestier',
			'Logique',
			'Outils',
		])
	})

	it('slots the collections category in second place when asked to', () => {
		const categories = creatorCategories(descriptors, { collections: true })

		expect(categories.map((item) => item.label)).toEqual([
			'Actions Mestier',
			'Collections',
			'Logique',
			'Outils',
		])
		expect(categories[1]).toMatchObject({
			id: COLLECTIONS_CATEGORY_ID,
			description: 'Appeler une API importée depuis une spec OpenAPI',
			kinds: [],
		})
	})

	it('describes each category as the prototype does', () => {
		expect(
			creatorCategories(descriptors).map((item) => item.description),
		).toEqual([
			'Agir dans Mestier : projets, tickets, devis, discussions',
			'Brancher, fusionner ou mettre en pause le workflow',
			'Exécuter du code, envoyer un e-mail, appeler une API',
		])
	})

	it('never offers an Odoo category: the Odoo types live among the tools', () => {
		const categories = creatorCategories(descriptors)
		const tools = categories.find((item) => item.label === 'Outils')

		expect(categories.map((item) => item.label)).not.toContain('Odoo')
		expect(tools?.kinds.map((kind) => kind.label)).toEqual([
			'Requête HTTP',
			'Créer un partenaire Odoo',
			'Créer une facture Odoo',
		])
	})

	it('hides the internal connector from the creator', () => {
		const actions = creatorCategories(descriptors)[0]

		expect(actions.kinds.map((kind) => kind.label)).not.toContain(
			'Étendre l’horizon de récurrence',
		)
	})

	it('gives each kind its own description instead of its category name', () => {
		const actions = creatorCategories(descriptors)[0]

		expect(actions.kinds.map((kind) => kind.description)).toEqual([
			'Crée un client dans Mestier',
			'Assigne un ticket à un membre de l’équipe',
			'Ajoute un badge de couleur à un ticket',
		])
	})

	it('lists the kinds in prototype order, whatever order the server answers in', () => {
		const forward = creatorCategories(descriptors)
		const backward = creatorCategories([...descriptors].reverse())

		expect(forward.map((item) => item.kinds.map((kind) => kind.label))).toEqual(
			[
				['Créer un client', 'Assigner un ticket', 'Ajouter un badge au ticket'],
				['Si', 'Pour chaque', 'Configuration'],
				['Requête HTTP', 'Créer un partenaire Odoo', 'Créer une facture Odoo'],
			],
		)
		expect(backward).toEqual(forward)
	})

	it('breaks ties between unknown kinds by label', () => {
		const alpha = descriptor('x.alpha', 'x', 'Alpha')
		const beta = descriptor('x.beta', 'x', 'Bêta')

		const [tools] = creatorCategories([beta, alpha])

		expect(tools.kinds.map((kind) => kind.label)).toEqual(['Alpha', 'Bêta'])
	})

	it('takes the icon and hue of each kind', () => {
		const actions = creatorCategories(descriptors)[0]

		expect(new Set(actions.kinds.map((kind) => kind.hue)).size).toBe(3)
	})
})

describe('triggersCategory', () => {
	it('offers the event trigger then the click, each with its own description', () => {
		const category = triggersCategory()

		expect(category.id).toBe(TRIGGERS_CATEGORY_ID)
		expect(category.kinds.map((kind) => kind.label)).toEqual([
			'Event Mestier',
			'Clic',
		])
		expect(category.kinds.map((kind) => kind.description)).toEqual([
			'Se déclenche quand un événement se produit dans Mestier',
			'Se déclenche quand vous cliquez sur « Tester le workflow »',
		])
	})
})

describe('searchCreatorKinds', () => {
	it('keeps only the matching kinds, accent-insensitively, across triggers too', () => {
		const result = searchCreatorKinds(allCategories(descriptors), 'creer')
		expect(
			result.flatMap((category) => category.kinds.map((k) => k.label)),
		).toEqual([
			'Créer un client',
			'Créer un partenaire Odoo',
			'Créer une facture Odoo',
		])
		expect(searchCreatorKinds(allCategories(descriptors), 'clic')[0].id).toBe(
			'triggers',
		)
	})

	it('searches the description as well as the label', () => {
		const result = searchCreatorKinds(allCategories(descriptors), 'couleur')

		expect(
			result.flatMap((category) => category.kinds.map((kind) => kind.label)),
		).toEqual(['Ajouter un badge au ticket'])
	})

	it('does not match a kind on the name of its category', () => {
		expect(searchCreatorKinds(allCategories(descriptors), 'logique')).toEqual(
			[],
		)
	})

	it('returns nothing for an unmatched query', () => {
		expect(searchCreatorKinds(allCategories(descriptors), 'zzz')).toEqual([])
	})
})

describe('allCategories', () => {
	it('puts the triggers first, then the connector categories', () => {
		expect(allCategories(descriptors).map((item) => item.label)).toEqual([
			'Déclencheurs',
			'Actions Mestier',
			'Logique',
			'Outils',
		])
	})
})
