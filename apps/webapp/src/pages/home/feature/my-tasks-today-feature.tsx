import { useEffect, useState } from 'react'
import {
	useCurrentTimeEntry,
	useDeclareTimeEntry,
	useMyFieldTasks,
	useStartTimeEntry,
	useStopTimeEntry,
} from '#/hooks/use-field'
import { fieldErrorMessage } from '#/pages/field/field-errors'
import {
	instantFromTimeInput,
	runningTask,
	timeInputValue,
} from '#/pages/field/types'
import { MyTasksTodayUI } from '#/pages/home/ui/my-tasks-today-ui'

/** Shown when a job switch leaves the worker unclocked, distinct from the
 * generic mutation error: the shared banner would only say the start failed,
 * and not that the earlier stop already went through. Mirrors the same
 * message on the field screen — same underlying operation, same failure. */
const SWITCH_ROLLBACK_MESSAGE =
	'Vous êtes maintenant décroché de tout chantier — réessayez de démarrer.'

interface MyTasksTodayFeatureProps {
	organizationId: string
}

/**
 * The dashboard's self-service pointage card.
 *
 * Every `/field/*` action is already scoped server-side to the caller's own
 * identity — nothing in the request body can target another employee — so
 * surfacing "start/stop my task" here, in the main panel, needed no backend
 * change. Only the on-site specifics (photo capture, day-end, recovering a
 * forgotten clock-off) stay behind on the dedicated `/field` screen; this
 * card covers the common case of validating a task from the desk.
 */
export function MyTasksTodayFeature({
	organizationId,
}: MyTasksTodayFeatureProps) {
	const tasks = useMyFieldTasks(organizationId)
	const current = useCurrentTimeEntry(organizationId)
	const startEntry = useStartTimeEntry(organizationId)
	const stopEntry = useStopTimeEntry(organizationId)
	const declareEntry = useDeclareTimeEntry(organizationId)

	// The running clock has to advance on its own, so the card ticks once a
	// minute. Seconds would be noise on a job measured in hours.
	const [now, setNow] = useState(() => Date.now())
	useEffect(() => {
		const timer = window.setInterval(() => setNow(Date.now()), 60_000)
		return () => window.clearInterval(timer)
	}, [])

	const [pendingTaskId, setPendingTaskId] = useState<string | null>(null)
	const [switchRollbackMessage, setSwitchRollbackMessage] = useState<
		string | null
	>(null)

	// At most one task's declare form open at a time, same discipline as the
	// rest of the card. Defaults span the last hour so a rush-hour worker
	// filling this in right after the fact has less to type.
	const [declaringTaskId, setDeclaringTaskId] = useState<string | null>(null)
	const [declareStart, setDeclareStart] = useState(() =>
		timeInputValue(new Date(Date.now() - 60 * 60_000)),
	)
	const [declareEnd, setDeclareEnd] = useState(() => timeInputValue(new Date()))

	const taskList =
		(tasks.data as { data?: FieldTaskRow[] } | undefined)?.data ?? []
	const currentStatus = (
		current.data as { data?: { running?: FieldEntry | null } } | undefined
	)?.data
	const running = currentStatus?.running ?? null

	// Switching jobs is stop-then-start, because the API refuses a second open
	// entry. Done in this order so a failure leaves nothing running rather than
	// two things running — mirrors the field screen's own logic exactly.
	const startTask = async (taskId: string) => {
		setPendingTaskId(taskId)
		setSwitchRollbackMessage(null)
		try {
			if (running) {
				await stopEntry.mutateAsync({
					path: { time_entry_id: running.id },
				} as never)
				try {
					await startEntry.mutateAsync({
						path: { organization_id: organizationId },
						body: { task_id: taskId },
					} as never)
				} catch {
					setSwitchRollbackMessage(SWITCH_ROLLBACK_MESSAGE)
				}
			} else {
				await startEntry.mutateAsync({
					path: { organization_id: organizationId },
					body: { task_id: taskId },
				} as never)
			}
		} finally {
			setPendingTaskId(null)
		}
	}

	const toggleDeclare = (taskId: string | null) => {
		setDeclaringTaskId(taskId)
		if (taskId) {
			setDeclareStart(timeInputValue(new Date(Date.now() - 60 * 60_000)))
			setDeclareEnd(timeInputValue(new Date()))
		}
	}

	const submitDeclare = () => {
		if (!declaringTaskId) return
		const today = new Date()
		const started_at = instantFromTimeInput(declareStart, today)
		const ended_at = instantFromTimeInput(declareEnd, today)
		if (!started_at || !ended_at) return

		declareEntry
			.mutateAsync({
				path: { organization_id: organizationId },
				body: { task_id: declaringTaskId, started_at, ended_at },
			} as never)
			.then(() => setDeclaringTaskId(null))
			.catch(() => {
				// Surfaced reactively via `declareEntry.error` above; this catch
				// only keeps the rejection from going unhandled and the form open
				// so the employee can correct the times and retry.
			})
	}

	const rawMutationErrorMessage =
		startEntry.error?.message ??
		stopEntry.error?.message ??
		declareEntry.error?.message ??
		null
	const mutationError =
		switchRollbackMessage ??
		(rawMutationErrorMessage
			? fieldErrorMessage(rawMutationErrorMessage)
			: null)

	return (
		<MyTasksTodayUI
			tasks={taskList}
			isLoading={tasks.isLoading || current.isLoading}
			loadFailed={tasks.isError || current.isError}
			onRetry={() => {
				void tasks.refetch()
				void current.refetch()
			}}
			running={running}
			runningTaskTitle={runningTask(taskList, running)?.title ?? null}
			now={now}
			pendingTaskId={pendingTaskId}
			isStopping={stopEntry.isPending && pendingTaskId === null}
			error={mutationError}
			onStart={(taskId) => void startTask(taskId)}
			onStop={() => {
				if (!running) return
				stopEntry
					.mutateAsync({ path: { time_entry_id: running.id } } as never)
					.catch(() => {
						// Surfaced reactively via `stopEntry.error` above; this catch
						// only keeps the rejection from going unhandled.
					})
			}}
			declaringTaskId={declaringTaskId}
			declareStart={declareStart}
			declareEnd={declareEnd}
			isDeclaring={declareEntry.isPending}
			onToggleDeclare={toggleDeclare}
			onDeclareStartChange={setDeclareStart}
			onDeclareEndChange={setDeclareEnd}
			onDeclareSubmit={submitDeclare}
		/>
	)
}

type FieldEntry = import('#/hooks/use-field').TimeEntry
type FieldTaskRow = import('#/hooks/use-field').FieldTask
