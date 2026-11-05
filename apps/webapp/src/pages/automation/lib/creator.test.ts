import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	allCategories,
	creatorCategories,
	searchCreatorKinds,
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

const descriptors = [
	descriptor('http.request', 'http', 'Requête HTTP'),
	descriptor('mestier.create_project', 'mestier', 'Créer un projet'),
	descriptor('flow.if', 'flow', 'Condition'),
	descriptor('odoo.create', 'odoo', 'Créer dans Odoo'),
]

describe('creatorCategories', () => {
	it('orders the known families first and names them in French', () => {
		expect(creatorCategories(descriptors).map((item) => item.label)).toEqual([
			'Actions Mestier',
			'Logique',
			'Outils',
			'Odoo',
		])
	})

	it('lists the connectors of a family', () => {
		const [mestier] = creatorCategories(descriptors)
		expect(mestier.kinds.map((kind) => kind.label)).toEqual(['Créer un projet'])
	})
})

describe('triggersCategory', () => {
	it('offers a manual trigger and an event trigger', () => {
		expect(triggersCategory().kinds.map((kind) => kind.label)).toEqual([
			'Clic',
			'Event Mestier',
		])
	})
})

describe('searchCreatorKinds', () => {
	it('keeps only the matching kinds, accent-insensitively, across triggers too', () => {
		const result = searchCreatorKinds(allCategories(descriptors), 'creer')
		expect(
			result.flatMap((category) => category.kinds.map((k) => k.label)),
		).toEqual(['Créer un projet', 'Créer dans Odoo'])
		expect(searchCreatorKinds(allCategories(descriptors), 'clic')[0].id).toBe(
			'triggers',
		)
	})

	it('returns nothing for an unmatched query', () => {
		expect(searchCreatorKinds(allCategories(descriptors), 'zzz')).toEqual([])
	})
})
