import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ConnectorSearchList } from '#/pages/automation/ui/connector-search-list'

const FAMILIES = [
	{
		family: 'http',
		connectors: [
			{ kind: 'http.request', label: 'Requête HTTP' },
			{ kind: 'http.webhook', label: 'Webhook sortant' },
		],
	},
	{
		family: 'odoo',
		connectors: [{ kind: 'odoo.upsert', label: 'Écrire dans Odoo' }],
	},
]

describe('ConnectorSearchList — browsing', () => {
	it('lists every connector grouped by family with no search text', () => {
		render(<ConnectorSearchList families={FAMILIES} onSelect={vi.fn()} />)

		expect(screen.getByText('Requête HTTP')).toBeDefined()
		expect(screen.getByText('Webhook sortant')).toBeDefined()
		expect(screen.getByText('Écrire dans Odoo')).toBeDefined()
	})

	it('calls onSelect with the connector kind, not its label', async () => {
		const user = userEvent.setup()
		const onSelect = vi.fn()
		render(<ConnectorSearchList families={FAMILIES} onSelect={onSelect} />)

		await user.click(screen.getByText('Requête HTTP'))

		expect(onSelect).toHaveBeenCalledWith('http.request')
	})
})

describe('ConnectorSearchList — search', () => {
	it('filters by connector label', async () => {
		const user = userEvent.setup()
		render(<ConnectorSearchList families={FAMILIES} onSelect={vi.fn()} />)

		await user.type(
			screen.getByPlaceholderText('Rechercher un connecteur…'),
			'webhook',
		)

		expect(screen.getByText('Webhook sortant')).toBeDefined()
		expect(screen.queryByText('Requête HTTP')).toBeNull()
		expect(screen.queryByText('Écrire dans Odoo')).toBeNull()
	})

	it('filters by family name too', async () => {
		const user = userEvent.setup()
		render(<ConnectorSearchList families={FAMILIES} onSelect={vi.fn()} />)

		await user.type(
			screen.getByPlaceholderText('Rechercher un connecteur…'),
			'odoo',
		)

		expect(screen.getByText('Écrire dans Odoo')).toBeDefined()
		expect(screen.queryByText('Requête HTTP')).toBeNull()
	})

	it('shows a message instead of an empty list when nothing matches', async () => {
		const user = userEvent.setup()
		render(<ConnectorSearchList families={FAMILIES} onSelect={vi.fn()} />)

		await user.type(
			screen.getByPlaceholderText('Rechercher un connecteur…'),
			'nothing-matches-this',
		)

		expect(screen.getByText('Aucun connecteur ne correspond')).toBeDefined()
	})
})
