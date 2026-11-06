import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { OperationOutputPanel } from '#/pages/automation/ui/operation-output-panel'

describe('OperationOutputPanel', () => {
	it('shows the example of the response in the shape the node outputs', () => {
		render(
			<OperationOutputPanel
				summary="Modifier un animal"
				status="200"
				output={{
					status: 200,
					headers: { 'content-type': 'application/json' },
					body: { id: 7, name: 'Rex' },
				}}
			/>,
		)

		expect(
			screen.getByText(
				/Ce nœud n’a pas encore été exécuté\. Exemple de réponse 200 de/,
			),
		).toBeDefined()
		expect(screen.getByText('Modifier un animal')).toBeDefined()
		const shown = screen.getByRole('region', { name: 'Sortie' }).textContent
		expect(shown).toContain('"status": 200')
		expect(shown).toContain('"content-type"')
		expect(shown).toContain('"body"')
		expect(shown).toContain('"name": "Rex"')
		expect(screen.queryByText(/La spec ne décrit pas/)).toBeNull()
	})

	it('says so when the spec does not describe the body', () => {
		render(
			<OperationOutputPanel
				summary="Supprimer"
				status="200"
				output={{
					status: 200,
					headers: { 'content-type': 'application/json' },
					body: {},
				}}
			/>,
		)

		expect(screen.getByText(/La spec ne décrit pas le contenu/)).toBeDefined()
		expect(
			screen.getByRole('region', { name: 'Sortie' }).textContent,
		).toContain('"status": 200')
	})
})
