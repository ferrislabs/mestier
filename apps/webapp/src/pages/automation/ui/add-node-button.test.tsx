import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { AddNodeButton } from '#/pages/automation/ui/add-node-button'

function descriptor(
	kind: string,
	label: string,
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches: [],
		family: 'flow',
		fields: [],
		kind,
		label,
		output_example: null,
		version: 1,
	}
}

const SIMPLE = descriptor('test.simple', 'Étape simple')

describe('AddNodeButton', () => {
	it('opens the search list on click, and hides it again once a connector is chosen', async () => {
		const user = userEvent.setup()
		const onSelect = vi.fn()
		render(
			<AddNodeButton
				ariaLabel="Ajouter un connecteur"
				catalogue={[SIMPLE]}
				onSelect={onSelect}
			/>,
		)

		expect(screen.queryByText('Étape simple')).toBeNull()

		await user.click(
			screen.getByRole('button', { name: 'Ajouter un connecteur' }),
		)
		expect(await screen.findByText('Étape simple')).toBeDefined()

		await user.click(screen.getByText('Étape simple'))

		expect(onSelect).toHaveBeenCalledWith(SIMPLE)
		expect(screen.queryByText('Étape simple')).toBeNull()
	})
})
