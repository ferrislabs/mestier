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
		tasksLoadFailed: false,
		onRetryTasks: vi.fn(),
		running: null,
		currentLoadFailed: false,
		onRetryCurrent: vi.fn(),
		staleEntry: null,
		staleTaskTitle: null,
		recoverTime: '17:30',
		isRecovering: false,
		dayEndedAt: null,
		now: NOW,
		dayEndTime: '18:00',
		error: null,
		pendingTaskId: null,
		pendingPhotoPhase: null,
		isStopping: false,
		isEndingDay: false,
		onRecoverTimeChange: vi.fn(),
		onRecover: vi.fn(),
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

	/**
	 * The forgotten clock-off. Yesterday's stretch has to be settled before
	 * anything else, because the server will not let another job start while it
	 * is open, and it cannot be closed at today's time without inventing hours.
	 */
	it('asks when an unfinished stretch from an earlier day actually ended', async () => {
		const stale = entry({ started_at: '2026-08-18T06:00:00Z' })
		await renderWithRouter(
			<FieldDayUI
				{...baseProps()}
				running={stale}
				staleEntry={stale}
				staleTaskTitle="Taille de haie"
			/>,
		)

		expect(screen.getByText(/chantier non clôturé/i)).toBeDefined()
		expect(screen.getByLabelText(/heure de fin/i)).toBeDefined()
		expect(screen.getByRole('button', { name: /^clôturer$/i })).toBeDefined()
	})

	it('says why nothing else can start until it is settled', async () => {
		const stale = entry({ started_at: '2026-08-18T06:00:00Z' })
		await renderWithRouter(
			<FieldDayUI {...baseProps()} running={stale} staleEntry={stale} />,
		)

		expect(
			screen.getByText(/vous ne pouvez pas en démarrer un autre/i),
		).toBeDefined()
	})

	/** One decision at a time: the day's jobs come back once yesterday is closed. */
	it('hides the day list while an earlier stretch is open', async () => {
		const stale = entry({ started_at: '2026-08-18T06:00:00Z' })
		await renderWithRouter(
			<FieldDayUI {...baseProps()} running={stale} staleEntry={stale} />,
		)

		expect(screen.queryByText(/mes chantiers du jour/i)).toBeNull()
	})

	it('shows the normal running card when the stretch is from today', async () => {
		await renderWithRouter(
			<FieldDayUI {...baseProps()} running={entry()} staleEntry={null} />,
		)

		expect(screen.queryByText(/chantier non clôturé/i)).toBeNull()
		expect(screen.getByText(/en cours depuis/i)).toBeDefined()
	})

	/** A failed fetch and a genuinely empty day must not look the same. */
	it('offers a retry instead of the empty state when the task list failed to load', async () => {
		const props = { ...baseProps(), tasks: [], tasksLoadFailed: true }
		await renderWithRouter(<FieldDayUI {...props} />)

		expect(screen.getByText(/échec du chargement — réessayer/i)).toBeDefined()
		expect(screen.queryByText(/aucun chantier ne vous est assigné/i)).toBeNull()

		await userEvent.click(screen.getByRole('button', { name: /réessayer/i }))
		expect(props.onRetryTasks).toHaveBeenCalled()
	})

	it('warns when the current status failed to load, with a way to retry', async () => {
		const props = { ...baseProps(), currentLoadFailed: true }
		await renderWithRouter(<FieldDayUI {...props} />)

		expect(
			screen.getByText(/échec du chargement de votre pointage en cours/i),
		).toBeDefined()

		await userEvent.click(screen.getByRole('button', { name: /réessayer/i }))
		expect(props.onRetryCurrent).toHaveBeenCalled()
	})
})
