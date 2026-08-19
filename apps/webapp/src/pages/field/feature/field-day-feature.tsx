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
	useStartTimeEntry,
	useStopTimeEntry,
} from '#/hooks/use-field'
import { instantFromTimeInput, timeInputValue } from '#/pages/field/types'
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
	// Kept from the mutation's answer rather than read back: there is no route
	// for "today's day log", and adding one to show a confirmation the worker
	// just caused would be a round trip for nothing.
	const [dayEndedAt, setDayEndedAt] = useState<string | null>(null)
	const [pendingTaskId, setPendingTaskId] = useState<string | null>(null)
	const [pendingPhotoPhase, setPendingPhotoPhase] = useState<PhotoPhase | null>(
		null,
	)

	const running =
		(current.data as { data?: FieldEntry | null } | undefined)?.data ?? null
	const taskList =
		(tasks.data as { data?: FieldTaskRow[] } | undefined)?.data ?? []

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
			dayEndedAt={dayEndedAt}
			now={now}
			dayEndTime={dayEndTime}
			error={
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
