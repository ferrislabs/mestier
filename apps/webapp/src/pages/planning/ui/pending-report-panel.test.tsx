import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { AssignmentReport } from '#/hooks/use-assignment-reports'
import { minutesLabel } from '#/pages/planning/lib/pending-reports'
import { PendingReportPanel } from '#/pages/planning/ui/pending-report-panel'

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
		memberName: (id: string) => (id === 'member-1' ? 'Alice Dupont' : id),
		plannedLabel: '4 h 00',
		reportedLabel: minutesLabel,
		applyingReportId: null,
		onApply: vi.fn(),
		onCancelApply: vi.fn(),
		isResolving: false,
		resolveError: null,
		dismissingReportId: null,
		dismissNote: '',
		onStartDismiss: vi.fn(),
		onCancelDismiss: vi.fn(),
		onDismissNoteChange: vi.fn(),
		onConfirmDismiss: vi.fn(),
	}
}

describe('PendingReportPanel', () => {
	it('renders nothing when there is no pending report', () => {
		const { container } = render(
			<PendingReportPanel {...baseProps()} reports={[]} />,
		)

		expect(container.firstChild).toBeNull()
	})

	it('shows what was planned, what was reported, and who said so', () => {
		render(<PendingReportPanel {...baseProps()} />)

		expect(screen.getByText(/prévu : 4 h 00/i)).toBeDefined()
		expect(screen.getByText(/déclaré : 5 h 00/i)).toBeDefined()
		expect(screen.getByText(/par alice dupont/i)).toBeDefined()
		expect(screen.getByText(/chantier plus long que prévu/i)).toBeDefined()
	})

	it('clicking Appliquer arms the report and shows the confirmation sentence', async () => {
		const props = baseProps()
		render(<PendingReportPanel {...props} />)

		await userEvent.click(screen.getByRole('button', { name: /^appliquer$/i }))

		expect(props.onApply).toHaveBeenCalledWith(report())
	})

	it('the armed report shows the confirmation sentence instead of the action buttons', () => {
		render(<PendingReportPanel {...baseProps()} applyingReportId="report-1" />)

		expect(screen.getByText(/enregistrez pour confirmer/i)).toBeDefined()
		expect(screen.queryByRole('button', { name: /^appliquer$/i })).toBeNull()
	})

	it('starting a dismiss opens the note field and confirming calls back with the report', async () => {
		const props = { ...baseProps(), dismissingReportId: 'report-1' }
		render(<PendingReportPanel {...props} />)

		await userEvent.click(
			screen.getByRole('button', { name: /confirmer le rejet/i }),
		)

		expect(props.onConfirmDismiss).toHaveBeenCalledWith(report())
	})

	it('clicking Rejeter starts the dismiss flow for that report', async () => {
		const props = baseProps()
		render(<PendingReportPanel {...props} />)

		await userEvent.click(screen.getByRole('button', { name: /^rejeter$/i }))

		expect(props.onStartDismiss).toHaveBeenCalledWith(report())
	})

	it('shows the resolve error near the report it belongs to', () => {
		render(
			<PendingReportPanel
				{...baseProps()}
				resolveError="Ce signalement a déjà été traité."
			/>,
		)

		expect(screen.getByText(/ce signalement a déjà été traité/i)).toBeDefined()
	})
})
