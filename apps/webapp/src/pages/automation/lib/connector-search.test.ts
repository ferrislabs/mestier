import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { searchConnectors } from '#/pages/automation/lib/connector-search'

function descriptor(
	kind: string,
	label: string,
	family: string,
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches: [],
		family,
		fields: [],
		kind,
		label,
		output_example: null,
		version: 1,
	}
}

describe('searchConnectors', () => {
	it('groups the connectors by family', () => {
		const connectors = [
			descriptor('http.request', 'Requête HTTP', 'flow'),
			descriptor('odoo.invoice.create', 'Créer une facture', 'odoo'),
			descriptor('odoo.quote.create', 'Créer un devis', 'odoo'),
		]

		const groups = searchConnectors(connectors, '')

		expect(groups.map((group) => group.family)).toEqual(['flow', 'odoo'])
		expect(groups.find((group) => group.family === 'odoo')?.connectors).toEqual(
			[connectors[1], connectors[2]],
		)
	})

	it('filters on the label, case-insensitively', () => {
		const connectors = [
			descriptor('http.request', 'Requête HTTP', 'flow'),
			descriptor('odoo.invoice.create', 'Créer une facture', 'odoo'),
		]

		const groups = searchConnectors(connectors, 'FACTURE')

		expect(groups).toEqual([
			{ family: 'odoo', connectors: [connectors[1]] },
		])
	})

	it('does not match on the kind, only the label', () => {
		const connectors = [descriptor('odoo.invoice.create', 'Créer une facture', 'odoo')]

		expect(searchConnectors(connectors, 'invoice.create')).toEqual([])
	})

	it('is empty when nothing matches', () => {
		const connectors = [descriptor('http.request', 'Requête HTTP', 'flow')]

		expect(searchConnectors(connectors, 'inconnu')).toEqual([])
	})

	it('returns every connector when the query is blank', () => {
		const connectors = [descriptor('http.request', 'Requête HTTP', 'flow')]

		expect(searchConnectors(connectors, '  ')).toEqual([
			{ family: 'flow', connectors },
		])
	})
})
