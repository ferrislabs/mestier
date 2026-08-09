import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
	LabelPastilles,
	STATUS_LABELS,
} from '#/pages/planning/ui/task-row-parts'

describe('LabelPastilles', () => {
	it('renders one pastille per label', () => {
		render(
			<LabelPastilles
				labels={[
					{ id: 'l1', name: 'Réunion', color: '#2563EB' },
					{ id: 'l2', name: 'Déplacement', color: '#16A34A' },
				]}
			/>,
		)
		expect(screen.getByText('Réunion')).toBeDefined()
		expect(screen.getByText('Déplacement')).toBeDefined()
	})

	it('shows a placeholder when there are no labels', () => {
		render(<LabelPastilles labels={[]} />)
		expect(screen.getByText('—')).toBeDefined()
	})
})

describe('STATUS_LABELS', () => {
	it('has a French label for every task status', () => {
		expect(STATUS_LABELS.PLANNED).toBe('Planifiée')
		expect(STATUS_LABELS.IN_PROGRESS).toBe('En cours')
		expect(STATUS_LABELS.DONE).toBe('Terminée')
		expect(STATUS_LABELS.CANCELLED).toBe('Annulée')
	})
})

describe('task-row-parts — no network call', () => {
	let fetchSpy: ReturnType<typeof createFetchSpy>

	function createFetchSpy() {
		return vi.spyOn(global, 'fetch').mockImplementation(() => {
			throw new Error('le composant ui/ ne doit jamais appeler fetch')
		})
	}

	beforeEach(() => {
		fetchSpy = createFetchSpy()
	})

	afterEach(() => {
		fetchSpy.mockRestore()
	})

	it('never triggers fetch', () => {
		render(
			<LabelPastilles
				labels={[{ id: 'l1', name: 'Réunion', color: '#2563EB' }]}
			/>,
		)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
