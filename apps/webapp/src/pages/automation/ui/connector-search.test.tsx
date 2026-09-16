import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { ConnectorSearch } from '#/pages/automation/ui/connector-search'

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

const HTTP = descriptor('http.request', 'Requête HTTP', 'flow')
const INVOICE = descriptor('odoo.invoice.create', 'Créer une facture', 'odoo')
const QUOTE = descriptor('odoo.quote.create', 'Créer un devis', 'odoo')

describe('ConnectorSearch', () => {
	it('renders every connector grouped under its family', () => {
		render(
			<ConnectorSearch
				connectors={[HTTP, INVOICE, QUOTE]}
				onSelect={vi.fn()}
			/>,
		)

		expect(screen.getByText('flow')).toBeDefined()
		expect(screen.getByText('odoo')).toBeDefined()
		expect(screen.getByText('Requête HTTP')).toBeDefined()
		expect(screen.getByText('Créer une facture')).toBeDefined()
		expect(screen.getByText('Créer un devis')).toBeDefined()
	})

	it('filters the list as the query changes', async () => {
		const user = userEvent.setup()
		render(
			<ConnectorSearch
				connectors={[HTTP, INVOICE, QUOTE]}
				onSelect={vi.fn()}
			/>,
		)

		await user.type(screen.getByRole('textbox'), 'facture')

		expect(screen.getByText('Créer une facture')).toBeDefined()
		expect(screen.queryByText('Créer un devis')).toBeNull()
		expect(screen.queryByText('Requête HTTP')).toBeNull()
	})

	it('calls onSelect with the chosen descriptor', async () => {
		const user = userEvent.setup()
		const onSelect = vi.fn()
		render(<ConnectorSearch connectors={[HTTP, INVOICE]} onSelect={onSelect} />)

		await user.click(screen.getByText('Créer une facture'))

		expect(onSelect).toHaveBeenCalledWith(INVOICE)
	})

	it('says plainly when nothing matches instead of showing an empty list', async () => {
		const user = userEvent.setup()
		render(<ConnectorSearch connectors={[HTTP]} onSelect={vi.fn()} />)

		await user.type(screen.getByRole('textbox'), 'inconnu')

		expect(screen.getByText('Aucun connecteur')).toBeDefined()
	})
})
