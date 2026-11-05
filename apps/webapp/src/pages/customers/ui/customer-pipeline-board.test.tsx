import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
import { EMPTY_EXTRAS } from '#/pages/customers/feature/mocks/account-extras'
import { CustomerPipelineBoard } from '#/pages/customers/ui/customer-pipeline-board'

function customer(
	pipelineStage: CustomerPipelineStage,
	overrides: Partial<Customer> = {},
): Customer {
	return {
		id: `customer-${pipelineStage}`,
		organization_id: 'org-1',
		name: 'Mairie de Saint-Julien',
		email: null,
		phone: null,
		status: pipelineStage === 'WON' ? 'CLIENT' : 'PROSPECT',
		pipeline_stage: pipelineStage,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function baseProps() {
	return {
		canMove: true,
		extrasOf: () => EMPTY_EXTRAS,
		onMove: vi.fn(),
	}
}

async function openMenu(user: ReturnType<typeof userEvent.setup>) {
	await user.click(
		screen.getByRole('button', {
			name: 'Déplacer « Mairie de Saint-Julien » vers…',
		}),
	)
}

describe('CustomerPipelineBoard — moving a card', () => {
	it('lists every other stage, explicit Gagné and Perdu included, but not the current one', async () => {
		const user = userEvent.setup()
		render(
			<CustomerPipelineBoard
				{...baseProps()}
				customers={[customer('QUOTE_SENT')]}
			/>,
		)

		await openMenu(user)

		expect(screen.getByRole('menuitem', { name: 'Nouveau' })).toBeDefined()
		expect(screen.getByRole('menuitem', { name: 'Contacté' })).toBeDefined()
		expect(screen.getByRole('menuitem', { name: 'Qualifié' })).toBeDefined()
		expect(screen.getByRole('menuitem', { name: 'Gagné' })).toBeDefined()
		expect(screen.getByRole('menuitem', { name: 'Perdu' })).toBeDefined()
		expect(screen.queryByRole('menuitem', { name: 'Devis envoyé' })).toBeNull()
	})

	it('calls onMove with WON when Gagné is picked', async () => {
		const user = userEvent.setup()
		const onMove = vi.fn()
		const target = customer('QUOTE_SENT')
		render(
			<CustomerPipelineBoard
				{...baseProps()}
				onMove={onMove}
				customers={[target]}
			/>,
		)

		await openMenu(user)
		await user.click(screen.getByRole('menuitem', { name: 'Gagné' }))

		expect(onMove).toHaveBeenCalledWith(target, 'WON')
	})

	it('calls onMove with LOST when Perdu is picked', async () => {
		const user = userEvent.setup()
		const onMove = vi.fn()
		const target = customer('CONTACTED')
		render(
			<CustomerPipelineBoard
				{...baseProps()}
				onMove={onMove}
				customers={[target]}
			/>,
		)

		await openMenu(user)
		await user.click(screen.getByRole('menuitem', { name: 'Perdu' }))

		expect(onMove).toHaveBeenCalledWith(target, 'LOST')
	})
})

describe('CustomerPipelineBoard — columns', () => {
	it('shows the six stages with their empty hints', () => {
		render(<CustomerPipelineBoard {...baseProps()} customers={[]} />)

		expect(screen.getAllByRole('region').length).toBe(6)
		expect(screen.getByText('Aucun devis en attente de réponse.')).toBeDefined()
	})

	it('only renders the stages it is given', () => {
		render(
			<CustomerPipelineBoard
				{...baseProps()}
				stages={['NEW', 'WON']}
				customers={[]}
			/>,
		)

		expect(screen.getAllByRole('region').length).toBe(2)
	})
})

describe('CustomerPipelineBoard — without MANAGE_CUSTOMERS', () => {
	it('hides the move menu on a card', () => {
		render(
			<CustomerPipelineBoard
				{...baseProps()}
				canMove={false}
				customers={[customer('QUOTE_SENT')]}
			/>,
		)

		expect(
			screen.queryByRole('button', {
				name: 'Déplacer « Mairie de Saint-Julien » vers…',
			}),
		).toBeNull()
		expect(screen.getByRole('article').className).not.toContain('cursor-grab')
	})
})
