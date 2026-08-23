import { AlertCircle } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useOrganizationList } from '#/hooks/use-active-organization'
import { useUploadFile } from '#/hooks/use-customers'
import {
	type AssignmentReport,
	type PhotoPhase,
	useAmendAssignmentReport,
	useAttachFieldPhoto,
	useCurrentTimeEntry,
	useEndWorkingDay,
	useMyAssignmentReports,
	useMyFieldTasks,
	useRecoverTimeEntry,
	useReportAssignment,
	useStartTimeEntry,
	useStopTimeEntry,
	useWithdrawAssignmentReport,
} from '#/hooks/use-field'
import { fieldErrorMessage } from '#/pages/field/field-errors'
import {
	instantFromTimeInput,
	isFromAnEarlierDay,
	timeInputValue,
} from '#/pages/field/types'
import { FieldDayUI } from '#/pages/field/ui/field-day-ui'

/** Shown when a job switch leaves the worker unclocked, distinct from the
 * generic mutation error: the shared banner would only say the start failed,
 * and not that the earlier stop already went through. */
const SWITCH_ROLLBACK_MESSAGE =
	'Vous êtes maintenant décroché de tout projet — réessayez de démarrer.'

interface FieldDayFeatureProps {
	organizationSlug: string
}

export function FieldDayFeature({ organizationSlug }: FieldDayFeatureProps) {
	const organizations = useOrganizationList()
	const organization = organizations.find(
		(candidate) => candidate.slug === organizationSlug,
	)

	if (!organization) {
		return (
			<div className="mx-auto flex min-h-dvh max-w-md flex-col items-center justify-center gap-3 p-6 text-center">
				<AlertCircle className="size-8 text-destructive" />
				<p className="font-semibold">Entreprise introuvable</p>
				<p className="text-sm text-muted-foreground">
					Ce compte n'a pas accès à « {organizationSlug} ».
				</p>
			</div>
		)
	}

	return (
		<FieldDayWorkspace
			key={organization.id}
			organizationId={organization.id}
			organizationName={organization.name}
			clockEnabled={organization.field_clock_enabled}
		/>
	)
}

function FieldDayWorkspace({
	organizationId,
	organizationName,
	clockEnabled,
}: {
	organizationId: string
	organizationName: string
	clockEnabled: boolean
}) {
	const tasks = useMyFieldTasks(organizationId)
	// No point polling for a running entry the screen won't show or act on
	// — see `field-day-ui.tsx`'s own `clockEnabled` gating.
	const current = useCurrentTimeEntry(organizationId, clockEnabled)
	const startEntry = useStartTimeEntry(organizationId)
	const stopEntry = useStopTimeEntry(organizationId)
	const attachPhoto = useAttachFieldPhoto(organizationId)
	const uploadFile = useUploadFile()
	const endDay = useEndWorkingDay(organizationId)
	const assignmentReports = useMyAssignmentReports(organizationId)
	const reportAssignment = useReportAssignment(organizationId)
	const amendReport = useAmendAssignmentReport(organizationId)
	const withdrawReport = useWithdrawAssignmentReport(organizationId)

	// The running clock has to advance on its own, so the screen ticks once a
	// minute. Seconds would be noise on a job measured in hours.
	const [now, setNow] = useState(() => Date.now())
	useEffect(() => {
		const timer = window.setInterval(() => setNow(Date.now()), 60_000)
		return () => window.clearInterval(timer)
	}, [])

	const [dayEndTime, setDayEndTime] = useState(() => timeInputValue(new Date()))
	const [recoverTime, setRecoverTime] = useState('17:30')
	// Optimistic overlay only: the server's answer, carried on `/field/current`,
	// is the source of truth. This exists purely so the screen doesn't flicker
	// back to the clock-in form for the instant between the mutation resolving
	// and the query it invalidates refetching — a reload always falls back to
	// the query, never to this.
	const [optimisticDayEndedAt, setOptimisticDayEndedAt] = useState<
		string | null
	>(null)
	const [pendingTaskId, setPendingTaskId] = useState<string | null>(null)
	const [pendingPhotoPhase, setPendingPhotoPhase] = useState<PhotoPhase | null>(
		null,
	)
	const [switchRollbackMessage, setSwitchRollbackMessage] = useState<
		string | null
	>(null)

	// At most one report form open at a time, across the whole day — mirrors
	// `staleEntry`'s own single-form convention on this screen.
	const [editingAssignmentId, setEditingAssignmentId] = useState<string | null>(
		null,
	)
	const [draftMinutes, setDraftMinutes] = useState('')
	const [draftComment, setDraftComment] = useState('')
	const [withdrawingReportId, setWithdrawingReportId] = useState<string | null>(
		null,
	)

	const recoverEntry = useRecoverTimeEntry(organizationId)
	const currentStatus = (
		current.data as
			| { data?: { running?: FieldEntry | null; day_ended_at?: string | null } }
			| undefined
	)?.data
	const running = currentStatus?.running ?? null
	const dayEndedAt = currentStatus?.day_ended_at ?? optimisticDayEndedAt ?? null
	const taskList =
		(tasks.data as { data?: FieldTaskRow[] } | undefined)?.data ?? []
	const reportList =
		(assignmentReports.data as { data?: AssignmentReport[] } | undefined)
			?.data ?? []

	// A stretch begun before today is the forgotten clock-off. The server
	// refuses to close it at the current time, because that would record hours
	// nobody worked, so the screen asks the employee when they actually
	// finished.
	const staleEntry =
		running && isFromAnEarlierDay(running.started_at, now) ? running : null
	const staleTaskTitle = staleEntry
		? (taskList.find((task) => task.id === staleEntry.task_id)?.title ?? null)
		: null

	/// Switching jobs is stop-then-start, because the API refuses a second open
	/// entry. Done in this order so a failure leaves nothing running rather than
	/// two things running. If the start half fails after the stop half already
	/// went through, the worker is now unclocked and must be told so distinctly:
	/// the generic mutation-error banner would only mention the failed start.
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

	const capturePhoto = async (phase: PhotoPhase, file: File) => {
		if (!running) return
		setPendingPhotoPhase(phase)
		try {
			const uploaded = await uploadFile.mutateAsync(file)
			await attachPhoto.mutateAsync({
				path: { time_entry_id: running.id },
				body: { phase, storage_key: uploaded.data.key },
			} as never)
		} finally {
			setPendingPhotoPhase(null)
		}
	}

	const openReportForm = (
		taskAssignmentId: string,
		existing: AssignmentReport | null,
	) => {
		setEditingAssignmentId(taskAssignmentId)
		const prefillFrom = existing?.resolution === 'PENDING' ? existing : null
		setDraftMinutes(prefillFrom ? String(prefillFrom.reported_minutes) : '')
		setDraftComment(prefillFrom?.comment ?? '')
	}

	const closeReportForm = () => {
		setEditingAssignmentId(null)
		setDraftMinutes('')
		setDraftComment('')
	}

	// No optimistic write: the form only closes once the server has actually
	// answered, so a report the worker believes was filed and was not is not
	// possible — see the issue's own acceptance criterion.
	const submitReport = async () => {
		if (!editingAssignmentId) return
		const minutes = Number(draftMinutes)
		if (!Number.isFinite(minutes) || minutes < 0) return
		const comment = draftComment.trim() ? draftComment.trim() : null
		const existing = reportList.find(
			(report) =>
				report.task_assignment_id === editingAssignmentId &&
				report.resolution === 'PENDING',
		)

		try {
			if (existing) {
				await amendReport.mutateAsync({
					path: { assignment_report_id: existing.id },
					body: { reported_minutes: minutes, comment },
				} as never)
			} else {
				await reportAssignment.mutateAsync({
					path: {
						organization_id: organizationId,
						task_assignment_id: editingAssignmentId,
					},
					body: { reported_minutes: minutes, comment },
				} as never)
			}
			closeReportForm()
		} catch {
			// Surfaced reactively via `reportAssignment.error`/`amendReport.error`
			// below; this catch only keeps the rejection from going unhandled.
		}
	}

	const withdrawReportById = async (reportId: string) => {
		setWithdrawingReportId(reportId)
		try {
			await withdrawReport.mutateAsync({
				path: { assignment_report_id: reportId },
			} as never)
		} finally {
			setWithdrawingReportId(null)
		}
	}

	// Mutation failures only. Query failures (below) are a different situation
	// — nothing was attempted and nothing to blame on the worker's last action
	// — so they get their own, distinct treatment rather than sharing this
	// banner.
	const rawMutationErrorMessage =
		recoverEntry.error?.message ??
		startEntry.error?.message ??
		stopEntry.error?.message ??
		attachPhoto.error?.message ??
		uploadFile.error?.message ??
		endDay.error?.message ??
		withdrawReport.error?.message ??
		null
	const mutationError =
		switchRollbackMessage ??
		(rawMutationErrorMessage
			? fieldErrorMessage(rawMutationErrorMessage)
			: null)

	// Kept apart from `mutationError`: this one belongs to whichever report
	// form is open, not to the shared banner, so it never orphans itself next
	// to a form the worker already closed.
	const rawReportErrorMessage =
		reportAssignment.error?.message ?? amendReport.error?.message ?? null
	const reportError = rawReportErrorMessage
		? fieldErrorMessage(rawReportErrorMessage)
		: null

	// Whether the current entry could not be loaded at all: distinct from
	// there being nothing running, since the screen must not offer to start a
	// second job when it simply failed to ask whether one is already open.
	const currentLoadFailed = current.isError

	return (
		<FieldDayUI
			organizationName={organizationName}
			clockEnabled={clockEnabled}
			tasks={taskList}
			tasksLoadFailed={tasks.isError}
			onRetryTasks={() => void tasks.refetch()}
			running={running}
			currentLoadFailed={currentLoadFailed}
			onRetryCurrent={() => void current.refetch()}
			staleEntry={staleEntry}
			staleTaskTitle={staleTaskTitle}
			recoverTime={recoverTime}
			isRecovering={recoverEntry.isPending}
			dayEndedAt={dayEndedAt}
			now={now}
			dayEndTime={dayEndTime}
			error={mutationError}
			pendingTaskId={pendingTaskId}
			pendingPhotoPhase={pendingPhotoPhase}
			isStopping={stopEntry.isPending && pendingTaskId === null}
			isEndingDay={endDay.isPending}
			onRecoverTimeChange={setRecoverTime}
			onRecover={() => {
				if (!staleEntry) return
				// The declared time belongs to the day the stretch started, not to
				// today: that is the whole point of asking.
				const ended_at = instantFromTimeInput(
					recoverTime,
					new Date(staleEntry.started_at),
				)
				if (!ended_at) return
				recoverEntry
					.mutateAsync({
						path: { time_entry_id: staleEntry.id },
						body: { ended_at },
					} as never)
					.catch(() => {
						// Surfaced reactively via `recoverEntry.error` above; this
						// catch only keeps the rejection from going unhandled.
					})
			}}
			onStart={(taskId) => void startTask(taskId)}
			onStop={() => {
				if (running) {
					stopEntry
						.mutateAsync({
							path: { time_entry_id: running.id },
						} as never)
						.catch(() => {
							// Surfaced reactively via `stopEntry.error` above; this catch
							// only keeps the rejection from going unhandled.
						})
				}
			}}
			onCapturePhoto={(phase, file) => void capturePhoto(phase, file)}
			onDayEndTimeChange={setDayEndTime}
			onEndDay={() => {
				// A time the browser could not parse is left to the server rather
				// than sent wrong.
				const ended_at = instantFromTimeInput(dayEndTime, new Date())
				endDay
					.mutateAsync({
						path: { organization_id: organizationId },
						body: ended_at ? { ended_at } : {},
					} as never)
					.then((answer) => {
						setOptimisticDayEndedAt(
							(answer as { data?: { ended_at?: string } } | undefined)?.data
								?.ended_at ?? null,
						)
					})
					.catch(() => {
						// Surfaced reactively via `endDay.error` above; this catch only
						// keeps the rejection from going unhandled.
					})
			}}
			reports={reportList}
			editingAssignmentId={editingAssignmentId}
			draftMinutes={draftMinutes}
			draftComment={draftComment}
			isSubmittingReport={reportAssignment.isPending || amendReport.isPending}
			withdrawingReportId={withdrawingReportId}
			reportError={reportError}
			onOpenReportForm={openReportForm}
			onCancelReportForm={closeReportForm}
			onDraftMinutesChange={setDraftMinutes}
			onDraftCommentChange={setDraftComment}
			onSubmitReport={() => void submitReport()}
			onWithdrawReport={(reportId) => void withdrawReportById(reportId)}
		/>
	)
}

type FieldEntry = import('#/hooks/use-field').TimeEntry
type FieldTaskRow = import('#/hooks/use-field').FieldTask
