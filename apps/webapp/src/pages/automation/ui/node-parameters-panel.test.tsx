import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import {
	NodeParametersPanel,
	OperationField,
	type OperationSummary,
} from '#/pages/automation/ui/node-parameters-panel'
import { renderWithRouter } from '#/test/render-with-router'

const OPERATION: OperationSummary = {
	method: 'PUT',
	path: '/pets/{id}',
	summary: 'Modifier un animal',
	collectionTitle: 'Petstore',
	collectionHref: '/o/atelier-bois/automatisation/collections/c-1',
	baseUrl: 'https://pets.test',
}

describe('OperationField', () => {
	it('heads the node with the method, the path, the summary and the collection', async () => {
		await renderWithRouter(<OperationField operation={OPERATION} />)

		expect(screen.getByText('PUT')).toBeDefined()
		expect(screen.getByText('/pets/{id}')).toBeDefined()
		expect(screen.getByText('Modifier un animal')).toBeDefined()
		expect(
			screen.getByRole('link', { name: 'Petstore' }).getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/collections/c-1')
		expect(screen.getByText(/https:\/\/pets\.test/)).toBeDefined()
	})

	it('leaves out the summary a spec did not give', async () => {
		await renderWithRouter(
			<OperationField operation={{ ...OPERATION, summary: '' }} />,
		)

		expect(screen.queryByText('Modifier un animal')).toBeNull()
		expect(screen.getByText('/pets/{id}')).toBeDefined()
	})

	it('says the operation cannot be found once the collections are known', async () => {
		await renderWithRouter(<OperationField operation={null} />)

		expect(
			screen.getByText(
				'Opération introuvable : la collection a été supprimée ou modifiée.',
			),
		).toBeDefined()
	})

	it('says it is loading rather than that the operation is gone', async () => {
		await renderWithRouter(
			<OperationField operation={null} unavailable="loading" />,
		)

		expect(screen.getByText('Chargement de l’opération…')).toBeDefined()
		expect(screen.queryByText(/introuvable/)).toBeNull()
	})

	it('says the collection could not be read, not that the operation is gone', async () => {
		await renderWithRouter(
			<OperationField operation={null} unavailable="unreadable" />,
		)

		expect(
			screen.getByText('Impossible de lire la collection de cette opération.'),
		).toBeDefined()
		expect(screen.queryByText(/introuvable/)).toBeNull()
	})

	it('shows what the server said of the operation', async () => {
		await renderWithRouter(
			<OperationField
				operation={null}
				errors={['connector `c1` references unknown collection `x`']}
			/>,
		)

		expect(screen.getByRole('alert').textContent).toBe(
			'connector `c1` references unknown collection `x`',
		)
	})
})

describe('NodeParametersPanel', () => {
	it('gives the tip about expressions when a field takes one', async () => {
		await renderWithRouter(
			<NodeParametersPanel acceptsExpressions>
				<p>champs</p>
			</NodeParametersPanel>,
		)

		expect(screen.getByText(/glissez une variable/)).toBeDefined()
	})

	it('words its tip like the prototype, naming the node', async () => {
		await renderWithRouter(
			<NodeParametersPanel acceptsExpressions>
				<p>champs</p>
			</NodeParametersPanel>,
		)

		expect(
			screen.getByText(
				/ou écrivez une expression qui nomme le nœud, par exemple/,
			),
		).toBeDefined()
	})

	it('writes its example of expression with the name of a node', async () => {
		await renderWithRouter(
			<NodeParametersPanel acceptsExpressions>
				<p>champs</p>
			</NodeParametersPanel>,
		)

		expect(
			screen.getByText("{{ $('Nom du nœud').item.json.client.nom }}"),
		).toBeDefined()
	})

	it('keeps the tip back when no field takes an expression', async () => {
		await renderWithRouter(
			<NodeParametersPanel acceptsExpressions={false}>
				<p>champs</p>
			</NodeParametersPanel>,
		)

		expect(screen.queryByText(/glissez une variable/)).toBeNull()
	})
})
