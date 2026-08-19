import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { FieldTask, TimeEntry } from '#/hooks/use-field'
import { FieldDayUI } from '#/pages/field/ui/field-day-ui'
import { renderWithRouter } from '#/test/render-with-router'

const NOW = new Date('2026-08-19T10:10:00Z').getTime()

function task(overrides: Partial<FieldTask> = {}): FieldTask {
	return {
		id: 'task-1',
		title: 'Taille de haie',
		description: null,
		starts_at: '2026-08-19T06:00:00Z',
		ends_at: '2026-08-19T14:00:00Z',
		all_day: false,
		status: 'PLANNED',
		customer_id: null,
		customer_context_id: null,
		...overrides,
	}
}

function entry(overrides: Partial<TimeEntry> = {}): TimeEntry {
	return {
		id: 'entry-1',
		organization_id: 'org-1',
		task_id: 'task-1',
		employee_id: 'employee-1',
		started_at: '2026-08-19T08:00:00Z',
		ended_at: null,
		worked_minutes: null,
		photos: [],
		...overrides,
	}
}

function baseProps() {
	return {
		organizationName: 'Paysages Bonnal',
		tasks: [task()],
		running: null,
		dayEndedAt: null,
		now: NOW,
		dayEndTime: '18:00',
		error: null,
		pendingTaskId: null,
		pendingPhotoPhase: null,
		isStopping: false,
		isEndingDay: false,
		onStart: vi.fn(),
		onStop: vi.fn(),
		onCapturePhoto: vi.fn(),
		onDayEndTimeChange: vi.fn(),
		onEndDay: vi.fn(),
	}
}

describe('FieldDayUI', () => {
	it('offers to start a job when nothing is running', async () => {
		const props = { ...baseProps(), onStart: vi.fn() }
		await renderWithRouter(<FieldDayUI {...props} />)

		await userEvent.click(screen.getByRole('button', { name: /démarrer/i }))

		expect(props.onStart).toHaveBeenCalledWith('task-1')
	})

	/// The running clock is the one thing that moves while the worker is out, so
	/// it is asserted rather than eyeballed.
	it('shows how long the running job has been going', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} running={entry()} />)

		expect(screen.getByText(/en cours depuis 2 h 10/i)).toBeDefined()
	})

	it('replaces the start button on the running job and offers to close it', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} running={entry()} />)

		expect(screen.queryByRole('button', { name: /^démarrer$/i })).toBeNull()
		expect(
			screen.getByRole('button', { name: /clôturer ce chantier/i }),
		).toBeDefined()
	})

	/// Warns that ending the day closes the open job, which is the alert the
	/// field app owes a worker who forgot to clock off.
	it('warns that the running job will be closed by the day end', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} running={entry()} />)

		expect(
			screen.getByText(/le chantier en cours sera clôturé à cette heure/i),
		).toBeDefined()
	})

	it('says nothing about closing a job when none is running', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} />)

		expect(screen.queryByText(/sera clôturé à cette heure/i)).toBeNull()
	})

	it('starting another job while one runs reads as a switch, not a start', async () => {
		await renderWithRouter(
			<FieldDayUI
				{...baseProps()}
				tasks={[task(), task({ id: 'task-2', title: 'Tonte du parc' })]}
				running={entry()}
			/>,
		)

		expect(
			screen.getByRole('button', { name: /basculer sur ce chantier/i }),
		).toBeDefined()
	})

	it('confirms the day is over instead of offering to end it again', async () => {
		await renderWithRouter(
			<FieldDayUI {...baseProps()} dayEndedAt="2026-08-19T16:30:00Z" />,
		)

		expect(screen.getByText(/journée terminée à/i)).toBeDefined()
		expect(
			screen.queryByRole('button', { name: /terminer ma journée/i }),
		).toBeNull()
	})

	it('says so plainly when nothing is assigned', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} tasks={[]} />)

		expect(
			screen.getByText(/aucun chantier ne vous est assigné/i),
		).toBeDefined()
	})
})
