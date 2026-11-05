import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { AssignmentReport } from '#/hooks/use-assignment-reports'
import { AssignmentReportListUI } from '#/pages/planning/ui/assignment-report-list-ui'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
;(globalThis as any).ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

function report(overrides: Partial<AssignmentReport> = {}): AssignmentReport {
	return {
		id: 'report-1',
		organization_id: 'org-1',
		task_assignment_id: 'assignment-1',
		reported_minutes: 300,
		comment: 'Chantier plus long que prévu',
		reported_by: 'member-1',
		resolution: 'PENDING',
		resolved_by: null,
		resolved_at: null,
		resolution_note: null,
		created_at: '2026-08-19T14:00:00Z',
		updated_at: '2026-08-19T14:00:00Z',
		...overrides,
	}
}

function baseProps() {
	return {
		reports: [report()],
		pagination: null,
		page: 1,
		pageSize: 25,
		resolution: 'PENDING' as const,
		memberName: (id: string) => (id === 'member-1' ? 'Alix Martin' : id),
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		onPageChange: vi.fn(),
		onPageSizeChange: vi.fn(),
		onResolutionChange: vi.fn(),
	}
}

describe('AssignmentReportListUI', () => {
	it('lists a report with what was reported and by whom', () => {
		render(<AssignmentReportListUI {...baseProps()} />)

		expect(screen.getByText(/5 h 00 déclarées/i)).toBeDefined()
		expect(screen.getByText(/par alix martin/i)).toBeDefined()
	})

	it('says so plainly when the resolution has nothing pending', () => {
		render(<AssignmentReportListUI {...baseProps()} reports={[]} />)

		expect(screen.getByText(/aucun écart en attente/i)).toBeDefined()
	})

	it('changing the filter calls back with the new resolution and resets to page 1', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<AssignmentReportListUI {...props} page={3} />)

		await user.click(screen.getByRole('button', { name: /filtres/i }))
		await user.click(screen.getByRole('menuitemradio', { name: /appliqués/i }))

		expect(props.onResolutionChange).toHaveBeenCalledWith('APPLIED')
		expect(props.onPageChange).toHaveBeenCalledWith(1)
	})

	it('shows the resolved-by and the note on a resolved report', () => {
		const resolved = report({
			resolution: 'DISMISSED',
			resolved_by: 'member-2',
			resolution_note: 'Doublon',
		})
		render(
			<AssignmentReportListUI
				{...baseProps()}
				reports={[resolved]}
				memberName={(id) => (id === 'member-1' ? 'Alix Martin' : 'Bob Manager')}
			/>,
		)

		expect(screen.getByText(/rejetés par bob manager/i)).toBeDefined()
		expect(screen.getByText(/doublon/i)).toBeDefined()
	})
})
