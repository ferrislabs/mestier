import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
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
		draggedCustomerId: null,
		onDragStart: vi.fn(),
		onDragEnd: vi.fn(),
		onDropOnStage: vi.fn(),
		onMove: vi.fn(),
	}
}

describe('CustomerPipelineBoard — terminal stages', () => {
	it('offers no sequence arrow past QUOTE_SENT, but offers the two explicit actions', () => {
		render(
			<CustomerPipelineBoard
				{...baseProps()}
				customers={[customer('QUOTE_SENT')]}
			/>,
		)

		expect(screen.queryByRole('button', { name: 'Avancer' })).toBeNull()
		expect(screen.getByRole('button', { name: 'Reculer' })).toBeDefined()
		expect(screen.getByRole('button', { name: 'Marquer gagné' })).toBeDefined()
		expect(screen.getByRole('button', { name: 'Marquer perdu' })).toBeDefined()
	})

	it('offers no backward arrow before NEW, and still offers the two explicit actions', () => {
		render(
			<CustomerPipelineBoard {...baseProps()} customers={[customer('NEW')]} />,
		)

		expect(screen.queryByRole('button', { name: 'Reculer' })).toBeNull()
		expect(screen.getByRole('button', { name: 'Avancer' })).toBeDefined()
		expect(screen.getByRole('button', { name: 'Marquer gagné' })).toBeDefined()
	})

	it('shows neither the sequence arrows nor the mark actions on a WON card', () => {
		render(
			<CustomerPipelineBoard {...baseProps()} customers={[customer('WON')]} />,
		)

		expect(screen.queryByRole('button', { name: 'Reculer' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Avancer' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Marquer gagné' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Marquer perdu' })).toBeNull()
	})

	it('shows neither the sequence arrows nor the mark actions on a LOST card', () => {
		render(
			<CustomerPipelineBoard {...baseProps()} customers={[customer('LOST')]} />,
		)

		expect(screen.queryByRole('button', { name: 'Avancer' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Marquer gagné' })).toBeNull()
	})

	it('calls onMove with WON when "Marquer gagné" is clicked from a non-terminal card', async () => {
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

		await user.click(screen.getByRole('button', { name: 'Marquer gagné' }))

		expect(onMove).toHaveBeenCalledWith(target, 'WON')
	})

	it('calls onMove with LOST when "Marquer perdu" is clicked from a non-terminal card', async () => {
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

		await user.click(screen.getByRole('button', { name: 'Marquer perdu' }))

		expect(onMove).toHaveBeenCalledWith(target, 'LOST')
	})
})

describe('CustomerPipelineBoard — without MANAGE_CUSTOMERS', () => {
	it('hides every move action and disables dragging on a non-terminal card', () => {
		render(
			<CustomerPipelineBoard
				{...baseProps()}
				canMove={false}
				customers={[customer('QUOTE_SENT')]}
			/>,
		)

		expect(screen.queryByRole('button', { name: 'Avancer' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Reculer' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Marquer gagné' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Marquer perdu' })).toBeNull()
		expect(screen.getByRole('article').getAttribute('draggable')).toBe('false')
	})
})
