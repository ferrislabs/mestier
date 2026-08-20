import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { FieldTask, TimeEntry } from '#/hooks/use-field'
import { MyTasksTodayUI } from '#/pages/home/ui/my-tasks-today-ui'

const NOW = new Date('2026-08-20T10:10:00Z').getTime()

function task(overrides: Partial<FieldTask> = {}): FieldTask {
	return {
		id: 'task-1',
		title: 'Taille de haie',
		description: null,
		starts_at: '2026-08-20T06:00:00Z',
		ends_at: '2026-08-20T14:00:00Z',
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
		started_at: '2026-08-20T08:00:00Z',
		ended_at: null,
		worked_minutes: null,
		photos: [],
		...overrides,
	}
}

function baseProps() {
	return {
		tasks: [task()],
		isLoading: false,
		loadFailed: false,
		onRetry: vi.fn(),
		running: null,
		runningTaskTitle: null,
		now: NOW,
		pendingTaskId: null,
		isStopping: false,
		error: null,
		onStart: vi.fn(),
		onStop: vi.fn(),
		declaringTaskId: null,
		declareStart: '08:00',
		declareEnd: '09:00',
		isDeclaring: false,
		onToggleDeclare: vi.fn(),
		onDeclareStartChange: vi.fn(),
		onDeclareEndChange: vi.fn(),
		onDeclareSubmit: vi.fn(),
	}
}

describe('MyTasksTodayUI', () => {
	it('offers to start a task that is not running', () => {
		render(<MyTasksTodayUI {...baseProps()} />)

		expect(screen.getByText('Taille de haie')).toBeDefined()
		expect(screen.getByRole('button', { name: /Démarrer/ })).toBeDefined()
	})

	it('calls onStart with the task id when "Démarrer" is clicked', async () => {
		const user = userEvent.setup()
		const onStart = vi.fn()
		render(<MyTasksTodayUI {...baseProps()} onStart={onStart} />)

		await user.click(screen.getByRole('button', { name: /Démarrer/ }))

		expect(onStart).toHaveBeenCalledWith('task-1')
	})

	it('shows the running task with an elapsed time and a stop button', () => {
		render(
			<MyTasksTodayUI
				{...baseProps()}
				running={entry()}
				runningTaskTitle="Taille de haie"
			/>,
		)

		expect(screen.getAllByText('Taille de haie').length).toBeGreaterThan(0)
		expect(screen.getByText(/En cours depuis/)).toBeDefined()
		expect(screen.getByRole('button', { name: /Clôturer/ })).toBeDefined()
	})

	it('offers "Basculer" instead of "Démarrer" on the other tasks once one is running', () => {
		render(
			<MyTasksTodayUI
				{...baseProps()}
				tasks={[task(), task({ id: 'task-2', title: 'Débroussaillage' })]}
				running={entry()}
				runningTaskTitle="Taille de haie"
			/>,
		)

		expect(screen.getByRole('button', { name: /Basculer/ })).toBeDefined()
		expect(screen.queryByRole('button', { name: /^Démarrer/ })).toBeNull()
	})

	it('calls onStop when "Clôturer" is clicked', async () => {
		const user = userEvent.setup()
		const onStop = vi.fn()
		render(
			<MyTasksTodayUI
				{...baseProps()}
				running={entry()}
				runningTaskTitle="Taille de haie"
				onStop={onStop}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /Clôturer/ }))

		expect(onStop).toHaveBeenCalled()
	})

	it('shows an empty state when there is nothing planned today', () => {
		render(<MyTasksTodayUI {...baseProps()} tasks={[]} />)

		expect(screen.getByText("Aucun chantier prévu aujourd'hui.")).toBeDefined()
	})

	it('shows a retry action when loading failed', async () => {
		const user = userEvent.setup()
		const onRetry = vi.fn()
		render(
			<MyTasksTodayUI
				{...baseProps()}
				tasks={[]}
				loadFailed
				onRetry={onRetry}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /Réessayer/ }))

		expect(onRetry).toHaveBeenCalled()
	})

	it('surfaces a mutation error banner', () => {
		render(
			<MyTasksTodayUI
				{...baseProps()}
				error="Vous êtes déjà pointé sur un autre chantier."
			/>,
		)

		expect(
			screen.getByText('Vous êtes déjà pointé sur un autre chantier.'),
		).toBeDefined()
	})

	it('opens the declare form for a task when "Déclarer un temps" is clicked', async () => {
		const user = userEvent.setup()
		const onToggleDeclare = vi.fn()
		render(
			<MyTasksTodayUI {...baseProps()} onToggleDeclare={onToggleDeclare} />,
		)

		await user.click(screen.getByRole('button', { name: /Déclarer un temps/ }))

		expect(onToggleDeclare).toHaveBeenCalledWith('task-1')
	})

	it('shows the declare form with start/end inputs for the declaring task', () => {
		render(<MyTasksTodayUI {...baseProps()} declaringTaskId="task-1" />)

		expect(screen.getByLabelText('Début')).toHaveProperty('value', '08:00')
		expect(screen.getByLabelText('Fin')).toHaveProperty('value', '09:00')
	})

	it('does not show the declare form when no task is declaring', () => {
		render(<MyTasksTodayUI {...baseProps()} declaringTaskId={null} />)

		expect(screen.queryByLabelText('Début')).toBeNull()
	})

	it('calls onDeclareSubmit when "Valider" is clicked', async () => {
		const user = userEvent.setup()
		const onDeclareSubmit = vi.fn()
		render(
			<MyTasksTodayUI
				{...baseProps()}
				declaringTaskId="task-1"
				onDeclareSubmit={onDeclareSubmit}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Valider' }))

		expect(onDeclareSubmit).toHaveBeenCalled()
	})

	it('closes the declare form when "Annuler" is clicked', async () => {
		const user = userEvent.setup()
		const onToggleDeclare = vi.fn()
		render(
			<MyTasksTodayUI
				{...baseProps()}
				declaringTaskId="task-1"
				onToggleDeclare={onToggleDeclare}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(onToggleDeclare).toHaveBeenCalledWith(null)
	})
})
