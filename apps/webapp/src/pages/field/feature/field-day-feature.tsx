import { AlertCircle } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useOrganizationList } from '#/hooks/use-active-organization'
import { useUploadFile } from '#/hooks/use-customers'
import {
	type PhotoPhase,
	useAttachFieldPhoto,
	useCurrentTimeEntry,
	useEndWorkingDay,
	useMyFieldTasks,
	useRecoverTimeEntry,
	useStartTimeEntry,
	useStopTimeEntry,
} from '#/hooks/use-field'
import {
	instantFromTimeInput,
	isFromAnEarlierDay,
	timeInputValue,
} from '#/pages/field/types'
import { FieldDayUI } from '#/pages/field/ui/field-day-ui'

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
		/>
	)
}

function FieldDayWorkspace({
	organizationId,
	organizationName,
}: {
	organizationId: string
	organizationName: string
}) {
	const tasks = useMyFieldTasks(organizationId)
	const current = useCurrentTimeEntry(organizationId)
	const startEntry = useStartTimeEntry(organizationId)
	const stopEntry = useStopTimeEntry(organizationId)
	const attachPhoto = useAttachFieldPhoto(organizationId)
	const uploadFile = useUploadFile()
	const endDay = useEndWorkingDay(organizationId)

	// The running clock has to advance on its own, so the screen ticks once a
	// minute. Seconds would be noise on a job measured in hours.
	const [now, setNow] = useState(() => Date.now())
	useEffect(() => {
		const timer = window.setInterval(() => setNow(Date.now()), 60_000)
		return () => window.clearInterval(timer)
	}, [])

	const [dayEndTime, setDayEndTime] = useState(() => timeInputValue(new Date()))
	const [recoverTime, setRecoverTime] = useState('17:30')
	// Kept from the mutation's answer rather than read back: there is no route
	// for "today's day log", and adding one to show a confirmation the worker
	// just caused would be a round trip for nothing.
	const [dayEndedAt, setDayEndedAt] = useState<string | null>(null)
	const [pendingTaskId, setPendingTaskId] = useState<string | null>(null)
	const [pendingPhotoPhase, setPendingPhotoPhase] = useState<PhotoPhase | null>(
		null,
	)

	const recoverEntry = useRecoverTimeEntry(organizationId)
	const running =
		(current.data as { data?: FieldEntry | null } | undefined)?.data ?? null
	const taskList =
		(tasks.data as { data?: FieldTaskRow[] } | undefined)?.data ?? []

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
	/// two things running.
	const startTask = async (taskId: string) => {
		setPendingTaskId(taskId)
		try {
			if (running) {
				await stopEntry.mutateAsync({
					path: { time_entry_id: running.id },
				} as never)
			}
			await startEntry.mutateAsync({
				path: { organization_id: organizationId },
				body: { task_id: taskId },
			} as never)
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

	return (
		<FieldDayUI
			organizationName={organizationName}
			tasks={taskList}
			running={running}
			staleEntry={staleEntry}
			staleTaskTitle={staleTaskTitle}
			recoverTime={recoverTime}
			isRecovering={recoverEntry.isPending}
			dayEndedAt={dayEndedAt}
			now={now}
			dayEndTime={dayEndTime}
			error={
				recoverEntry.error?.message ??
				startEntry.error?.message ??
				stopEntry.error?.message ??
				attachPhoto.error?.message ??
				uploadFile.error?.message ??
				endDay.error?.message ??
				tasks.error?.message ??
				current.error?.message ??
				null
			}
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
				void recoverEntry.mutateAsync({
					path: { time_entry_id: staleEntry.id },
					body: { ended_at },
				} as never)
			}}
			onStart={(taskId) => void startTask(taskId)}
			onStop={() => {
				if (running) {
					void stopEntry.mutateAsync({
						path: { time_entry_id: running.id },
					} as never)
				}
			}}
			onCapturePhoto={(phase, file) => void capturePhoto(phase, file)}
			onDayEndTimeChange={setDayEndTime}
			onEndDay={() => {
				// A time the browser could not parse is left to the server rather
				// than sent wrong.
				const ended_at = instantFromTimeInput(dayEndTime, new Date())
				void endDay
					.mutateAsync({
						path: { organization_id: organizationId },
						body: ended_at ? { ended_at } : {},
					} as never)
					.then((answer) => {
						setDayEndedAt(
							(answer as { data?: { ended_at?: string } } | undefined)?.data
								?.ended_at ?? null,
						)
					})
			}}
		/>
	)
}

type FieldEntry = import('#/hooks/use-field').TimeEntry
type FieldTaskRow = import('#/hooks/use-field').FieldTask
