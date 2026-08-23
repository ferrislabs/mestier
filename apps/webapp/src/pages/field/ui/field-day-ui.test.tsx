import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { AssignmentReport, FieldTask, TimeEntry } from '#/hooks/use-field'
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
		task_assignment_id: 'assignment-1',
		...overrides,
	}
}

function assignmentReport(
	overrides: Partial<AssignmentReport> = {},
): AssignmentReport {
	return {
		id: 'report-1',
		organization_id: 'org-1',
		task_assignment_id: 'assignment-1',
		reported_minutes: 300,
		comment: null,
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
		// Every existing test in this suite predates the correction-loop
		// clock demotion and exercises the clock itself, so the default here
		// keeps it on; the dedicated `clockEnabled: false` describe block
		// below covers the demoted screen.
		clockEnabled: true,
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
		reports: [] as AssignmentReport[],
		editingAssignmentId: null,
		draftMinutes: '',
		draftComment: '',
		isSubmittingReport: false,
		withdrawingReportId: null,
		reportError: null,
		onOpenReportForm: vi.fn(),
		onCancelReportForm: vi.fn(),
		onDraftMinutesChange: vi.fn(),
		onDraftCommentChange: vi.fn(),
		onSubmitReport: vi.fn(),
		onWithdrawReport: vi.fn(),
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
			screen.getByRole('button', { name: /clôturer ce projet/i }),
		).toBeDefined()
	})

	/// Warns that ending the day closes the open job, which is the alert the
	/// field app owes a worker who forgot to clock off.
	it('warns that the running job will be closed by the day end', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} running={entry()} />)

		expect(
			screen.getByText(/le projet en cours sera clôturé à cette heure/i),
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
			screen.getByRole('button', { name: /basculer sur ce projet/i }),
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

		expect(screen.getByText(/aucun projet ne vous est assigné/i)).toBeDefined()
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

		expect(screen.getByText(/projet non clôturé/i)).toBeDefined()
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

		expect(screen.queryByText(/mes projets du jour/i)).toBeNull()
	})

	it('shows the normal running card when the stretch is from today', async () => {
		await renderWithRouter(
			<FieldDayUI {...baseProps()} running={entry()} staleEntry={null} />,
		)

		expect(screen.queryByText(/projet non clôturé/i)).toBeNull()
		expect(screen.getByText(/en cours depuis/i)).toBeDefined()
	})

	/** A failed fetch and a genuinely empty day must not look the same. */
	it('offers a retry instead of the empty state when the task list failed to load', async () => {
		const props = { ...baseProps(), tasks: [], tasksLoadFailed: true }
		await renderWithRouter(<FieldDayUI {...props} />)

		expect(screen.getByText(/échec du chargement — réessayer/i)).toBeDefined()
		expect(screen.queryByText(/aucun projet ne vous est assigné/i)).toBeNull()

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

describe('FieldDayUI — signaler un écart', () => {
	/** The planned figure is always visible: a report is a comparison, and a
	 * form that hides what it compares against gets guessed at. */
	it('shows the planned duration next to a way to file a report', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} />)

		expect(screen.getByText(/prévu : 8 h 00/i)).toBeDefined()
		expect(
			screen.getByRole('button', { name: /signaler un écart/i }),
		).toBeDefined()
	})

	it('opening the form names the assignment and the existing report, if any', async () => {
		const props = { ...baseProps(), onOpenReportForm: vi.fn() }
		await renderWithRouter(<FieldDayUI {...props} />)

		await userEvent.click(
			screen.getByRole('button', { name: /signaler un écart/i }),
		)

		expect(props.onOpenReportForm).toHaveBeenCalledWith('assignment-1', null)
	})

	it('a pending report shows as pending, with a way to amend or withdraw it', async () => {
		await renderWithRouter(
			<FieldDayUI {...baseProps()} reports={[assignmentReport()]} />,
		)

		expect(screen.getByText(/en attente de validation/i)).toBeDefined()
		expect(screen.getByText(/déclaré : 5 h 00/i)).toBeDefined()
		expect(screen.getByRole('button', { name: /^modifier$/i })).toBeDefined()
		expect(screen.getByRole('button', { name: /^retirer$/i })).toBeDefined()
	})

	it('withdrawing calls back with the report id', async () => {
		const props = {
			...baseProps(),
			reports: [assignmentReport()],
			onWithdrawReport: vi.fn(),
		}
		await renderWithRouter(<FieldDayUI {...props} />)

		await userEvent.click(screen.getByRole('button', { name: /^retirer$/i }))

		expect(props.onWithdrawReport).toHaveBeenCalledWith('report-1')
	})

	it("shows a resolved report's decision and the manager's note", async () => {
		const resolved = assignmentReport({
			resolution: 'APPLIED',
			resolved_by: 'manager-1',
			resolved_at: '2026-08-19T16:00:00Z',
			resolution_note: 'Écart confirmé sur place',
		})
		await renderWithRouter(<FieldDayUI {...baseProps()} reports={[resolved]} />)

		expect(screen.getByText(/écart appliqué au planning/i)).toBeDefined()
		expect(screen.getByText(/écart confirmé sur place/i)).toBeDefined()
		expect(
			screen.getByRole('button', { name: /signaler un nouvel écart/i }),
		).toBeDefined()
	})

	it('reporting zero is phrased as the job not having happened, not as a zero duration', async () => {
		const props = {
			...baseProps(),
			editingAssignmentId: 'assignment-1',
			draftMinutes: '0',
		}
		await renderWithRouter(<FieldDayUI {...props} />)

		expect(
			screen.getByText(/vous déclarez que ce projet n'a pas eu lieu/i),
		).toBeDefined()
	})

	it('a resolved report shown as reported also uses the "did not happen" phrasing at zero', async () => {
		const resolved = assignmentReport({
			reported_minutes: 0,
			resolution: 'DISMISSED',
			resolved_by: 'manager-1',
			resolved_at: '2026-08-19T16:00:00Z',
		})
		await renderWithRouter(<FieldDayUI {...baseProps()} reports={[resolved]} />)

		expect(screen.getByText(/le projet n'a pas eu lieu/i)).toBeDefined()
	})

	it('submitting the open form calls back, and typing updates the drafts', async () => {
		const props = {
			...baseProps(),
			editingAssignmentId: 'assignment-1',
			draftMinutes: '180',
			onDraftMinutesChange: vi.fn(),
			onSubmitReport: vi.fn(),
		}
		await renderWithRouter(<FieldDayUI {...props} />)

		await userEvent.type(screen.getByLabelText(/durée réelle/i), '5')
		expect(props.onDraftMinutesChange).toHaveBeenCalled()

		await userEvent.click(screen.getByRole('button', { name: /^déclarer$/i }))
		expect(props.onSubmitReport).toHaveBeenCalled()
	})
})

describe('FieldDayUI — pointeuse désactivée', () => {
	it('shows the day’s tasks and their report control without any clock', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} clockEnabled={false} />)

		expect(screen.getByText('Taille de haie')).toBeDefined()
		expect(
			screen.getByRole('button', { name: /signaler un écart/i }),
		).toBeDefined()
	})

	it('offers no way to start a job', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} clockEnabled={false} />)

		expect(screen.queryByRole('button', { name: /^démarrer$/i })).toBeNull()
	})

	it('shows no running-job card even when the server reports one running', async () => {
		await renderWithRouter(
			<FieldDayUI {...baseProps()} clockEnabled={false} running={entry()} />,
		)

		expect(screen.queryByText(/en cours depuis/i)).toBeNull()
		expect(
			screen.queryByRole('button', { name: /clôturer ce projet/i }),
		).toBeNull()
	})

	it('shows no forgotten-clock-off prompt even for a stale entry', async () => {
		const stale = entry({ started_at: '2026-08-18T06:00:00Z' })
		await renderWithRouter(
			<FieldDayUI
				{...baseProps()}
				clockEnabled={false}
				running={stale}
				staleEntry={stale}
			/>,
		)

		expect(screen.queryByText(/projet non clôturé/i)).toBeNull()
		// The task list still shows, unlike the clock-enabled case where a
		// stale entry hides it until settled.
		expect(screen.getByText('Taille de haie')).toBeDefined()
	})

	it('shows no day-end section', async () => {
		await renderWithRouter(<FieldDayUI {...baseProps()} clockEnabled={false} />)

		expect(screen.queryByText(/fin de journée/i)).toBeNull()
		expect(
			screen.queryByRole('button', { name: /terminer ma journée/i }),
		).toBeNull()
	})
})
