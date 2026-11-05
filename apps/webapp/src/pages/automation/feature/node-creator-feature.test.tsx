import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	readCollections,
	removeCollectionMock,
} from '#/pages/automation/feature/mocks/collections-store'
import { NodeCreatorFeature } from '#/pages/automation/feature/node-creator-feature'
import { renderWithRouter } from '#/test/render-with-router'

function textField(name: string): Schemas.FieldResponse {
	return {
		expression: false,
		kind: 'Text',
		label: name,
		name,
		required: false,
		secret: false,
		visible_when: null,
	}
}

const HTTP_DESCRIPTOR: Schemas.ConnectorDescriptorResponse = {
	kind: 'http.request',
	family: 'http',
	label: 'Requête HTTP',
	version: 1,
	auth: 'None',
	fields: [textField('method'), textField('url')],
	branches: [],
	output_example: null,
}

function descriptorOf(
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

const CATALOGUE: Schemas.ConnectorDescriptorResponse[] = [
	descriptorOf('odoo.create_invoice', 'odoo', 'Créer une facture Odoo'),
	HTTP_DESCRIPTOR,
	descriptorOf('flow.condition', 'flow', 'Condition'),
	descriptorOf(
		'mestier.task_recurrence.extend_horizon',
		'mestier',
		'Étendre l’horizon de récurrence',
	),
	descriptorOf('mestier.support.add_badge', 'mestier', 'Ajouter un badge'),
	descriptorOf('mestier.customer.create', 'mestier', 'Créer un client'),
]

async function renderCreator(hasTrigger: boolean) {
	await renderWithRouter(
		<NodeCreatorFeature
			organizationSlug="atelier-bois"
			descriptors={CATALOGUE}
			hasTrigger={hasTrigger}
			onCreate={vi.fn()}
			onClose={vi.fn()}
		/>,
	)
	return screen.getByRole('complementary', { name: 'Créateur de nœuds' })
}

function rowsOf(panel: HTMLElement): string[] {
	return within(panel)
		.getAllByRole('listitem')
		.map((item) => item.textContent ?? '')
}

describe('NodeCreatorFeature — the catalogue it offers', () => {
	it('lists the prototype categories in order, then the trigger entry, with no Odoo or empty category', async () => {
		const panel = await renderCreator(true)

		expect(rowsOf(panel)).toEqual([
			'Actions MestierAgir dans Mestier : projets, tickets, devis, discussions',
			'CollectionsAppeler une API importée depuis une spec OpenAPI',
			'LogiqueBrancher, fusionner ou mettre en pause le workflow',
			'OutilsExécuter du code, envoyer un e-mail, appeler une API',
		])
		expect(
			within(panel).getByRole('button', { name: /Ajouter un déclencheur/ }),
		).toBeDefined()
		expect(within(panel).queryByText('Odoo')).toBeNull()
		expect(within(panel).queryByText('Validation humaine')).toBeNull()
	})

	it('shows each type with its own description, the Odoo ones among the tools', async () => {
		const user = userEvent.setup()
		const panel = await renderCreator(true)

		await user.click(within(panel).getByRole('button', { name: /Outils/ }))

		expect(rowsOf(panel)).toEqual([
			'Requête HTTPAppelle une API externe',
			'Créer une facture OdooCrée une facture client dans Odoo',
		])
	})

	it('never offers the internal horizon extension', async () => {
		const user = userEvent.setup()
		const panel = await renderCreator(true)

		await user.click(
			within(panel).getByRole('button', { name: /Actions Mestier/ }),
		)

		expect(rowsOf(panel)).toEqual([
			'Créer un clientCrée un client dans Mestier',
			'Ajouter un badge au ticketAjoute un badge de couleur à un ticket',
		])
	})

	it('starts on the triggers when the workflow has none, event first', async () => {
		const panel = await renderCreator(false)

		expect(
			within(panel).getByText('Qu’est-ce qui déclenche ce workflow ?'),
		).toBeDefined()
		expect(rowsOf(panel)).toEqual([
			'Event MestierSe déclenche quand un événement se produit dans Mestier',
			'ClicSe déclenche quand vous cliquez sur « Tester le workflow »',
		])
	})

	it('searches the description as well as the label, grouped by category', async () => {
		const user = userEvent.setup()
		const panel = await renderCreator(true)

		await user.type(
			within(panel).getByRole('searchbox', { name: 'Rechercher un nœud' }),
			'couleur',
		)

		expect(
			within(panel).getByRole('region', { name: 'Actions Mestier' }),
		).toBeDefined()
		expect(rowsOf(panel)).toEqual([
			'Ajouter un badge au ticketAjoute un badge de couleur à un ticket',
		])
	})
})

describe('NodeCreatorFeature', () => {
	it('sends to the collections page when no collection has been imported', async () => {
		const user = userEvent.setup()
		for (const collection of readCollections()) {
			await removeCollectionMock(collection.id)
		}
		await renderWithRouter(
			<NodeCreatorFeature
				organizationSlug="atelier-bois"
				descriptors={[HTTP_DESCRIPTOR]}
				hasTrigger
				onCreate={vi.fn()}
				onClose={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /Collections/ }))

		const panel = screen.getByRole('complementary', {
			name: 'Créateur de nœuds',
		})
		expect(
			within(panel)
				.getByRole('link', { name: 'Importer une collection' })
				.getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/collections')
	})
})
