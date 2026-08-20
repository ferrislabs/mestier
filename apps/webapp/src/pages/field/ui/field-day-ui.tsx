import {
	AlertCircle,
	CheckCircle2,
	Loader2,
	MapPin,
	Play,
	Square,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import type { FieldTask, PhotoPhase, TimeEntry } from '#/hooks/use-field'
import {
	elapsedLabel,
	runningTask,
	startedAtLabel,
	taskWindowLabel,
} from '../types'
import { FieldPhotoPicker } from './field-photo-picker'

interface FieldDayUIProps {
	organizationName: string
	tasks: FieldTask[]
	running: TimeEntry | null
	/**
	 * The running stretch when it began before today: the forgotten clock-off.
	 *
	 * Handed down separately rather than derived here, because the server is the
	 * authority on which day a stretch belongs to and this file does no date
	 * arithmetic of its own.
	 */
	staleEntry: TimeEntry | null
	staleTaskTitle: string | null
	recoverTime: string
	isRecovering: boolean
	dayEndedAt: string | null
	/** Re-rendered every minute so the running clock advances. */
	now: number
	dayEndTime: string
	error: string | null
	pendingTaskId: string | null
	pendingPhotoPhase: PhotoPhase | null
	isStopping: boolean
	isEndingDay: boolean
	onRecoverTimeChange: (value: string) => void
	onRecover: () => void
	onStart: (taskId: string) => void
	onStop: () => void
	onCapturePhoto: (phase: PhotoPhase, file: File) => void
	onDayEndTimeChange: (value: string) => void
	onEndDay: () => void
}

/**
 * The whole field app, one screen.
 *
 * Written for a phone held in one gloved hand: a single column, controls at
 * least 48px tall, and the running job pinned at the top because it is the only
 * thing that changes while the worker is out. Nothing here fetches; the feature
 * above resolves everything and hands it down.
 */
export function FieldDayUI({
	organizationName,
	tasks,
	running,
	staleEntry,
	staleTaskTitle,
	recoverTime,
	isRecovering,
	dayEndedAt,
	now,
	dayEndTime,
	error,
	pendingTaskId,
	pendingPhotoPhase,
	isStopping,
	isEndingDay,
	onRecoverTimeChange,
	onRecover,
	onStart,
	onStop,
	onCapturePhoto,
	onDayEndTimeChange,
	onEndDay,
}: FieldDayUIProps) {
	const current = runningTask(tasks, running)

	return (
		<div className="mx-auto flex min-h-dvh w-full max-w-md flex-col gap-4 bg-muted/30 p-4 pb-8">
			<header className="flex items-baseline justify-between">
				<p className="text-lg font-bold">Ma journée</p>
				<p className="truncate text-sm text-muted-foreground">
					{organizationName}
				</p>
			</header>

			{error ? (
				<div className="flex items-start gap-3 rounded-xl border border-destructive/30 bg-destructive-soft p-4 text-destructive">
					<AlertCircle className="mt-0.5 size-5 shrink-0" />
					<p className="text-sm font-medium">{error}</p>
				</div>
			) : null}

			{staleEntry ? (
				<section className="rounded-xl border-2 border-amber-500 bg-card p-4">
					<p className="text-xs font-semibold uppercase tracking-wide text-amber-600 dark:text-amber-500">
						Chantier non clôturé
					</p>
					<p className="mt-1 text-xl font-bold leading-tight">
						{staleTaskTitle ?? 'Chantier'}
					</p>
					<p className="mt-1 text-sm text-muted-foreground">
						Démarré {startedAtLabel(staleEntry.started_at)}. À quelle heure
						avez-vous terminé&nbsp;?
					</p>

					<div className="mt-4 flex gap-2">
						<Input
							type="time"
							aria-label="Heure de fin"
							className="h-14 w-28 text-base"
							value={recoverTime}
							onChange={(event) => onRecoverTimeChange(event.target.value)}
						/>
						<Button
							type="button"
							size="lg"
							className="h-14 flex-1 text-base"
							disabled={isRecovering}
							onClick={onRecover}
						>
							{isRecovering ? <Loader2 className="animate-spin" /> : null}
							Clôturer
						</Button>
					</div>
					<p className="mt-2 text-xs text-muted-foreground">
						Tant que ce chantier n'est pas clôturé, vous ne pouvez pas en
						démarrer un autre.
					</p>
				</section>
			) : null}

			{running && !staleEntry ? (
				<section className="rounded-xl border-2 border-primary bg-card p-4">
					<p className="text-xs font-semibold uppercase tracking-wide text-primary">
						En cours depuis {elapsedLabel(running.started_at, now)}
					</p>
					<p className="mt-1 text-xl font-bold leading-tight">
						{current?.title ?? 'Chantier'}
					</p>

					<div className="mt-4">
						<FieldPhotoPicker
							photos={running.photos}
							pendingPhase={pendingPhotoPhase}
							onCapture={onCapturePhoto}
						/>
					</div>

					<Button
						type="button"
						size="lg"
						variant="destructive"
						className="mt-4 h-14 w-full text-base"
						disabled={isStopping}
						onClick={onStop}
					>
						{isStopping ? (
							<Loader2 className="animate-spin" />
						) : (
							<Square className="fill-current" />
						)}
						Clôturer ce chantier
					</Button>
				</section>
			) : null}

			{staleEntry ? null : (
				<section className="space-y-2">
					<p className="text-sm font-semibold text-muted-foreground">
						Mes chantiers du jour
					</p>

					{tasks.length === 0 ? (
						<p className="rounded-xl border bg-card p-4 text-sm text-muted-foreground">
							Aucun chantier ne vous est assigné aujourd'hui.
						</p>
					) : (
						tasks.map((task) => {
							const isCurrent = running?.task_id === task.id
							const window = taskWindowLabel(task)

							return (
								<div
									key={task.id}
									className="rounded-xl border bg-card p-4 data-[current=true]:border-primary"
									data-current={isCurrent}
								>
									<div className="flex items-start justify-between gap-3">
										<div className="min-w-0">
											<p className="font-semibold leading-tight">
												{task.title}
											</p>
											{window ? (
												<p className="mt-0.5 text-sm text-muted-foreground">
													{window}
												</p>
											) : null}
										</div>
										{isCurrent ? (
											<CheckCircle2 className="size-5 shrink-0 text-primary" />
										) : null}
									</div>

									{task.description ? (
										<p className="mt-2 whitespace-pre-wrap text-sm text-muted-foreground">
											{task.description}
										</p>
									) : null}

									{task.customer_context_id ? (
										<p className="mt-2 flex items-center gap-1.5 text-xs text-muted-foreground">
											<MapPin className="size-3.5 shrink-0" />
											Adresse renseignée sur le chantier
										</p>
									) : null}

									{!isCurrent ? (
										<Button
											type="button"
											size="lg"
											variant={running ? 'outline' : 'default'}
											className="mt-3 h-12 w-full"
											disabled={pendingTaskId !== null}
											onClick={() => onStart(task.id)}
										>
											{pendingTaskId === task.id ? (
												<Loader2 className="animate-spin" />
											) : (
												<Play className="fill-current" />
											)}
											{running ? 'Basculer sur ce chantier' : 'Démarrer'}
										</Button>
									) : null}
								</div>
							)
						})
					)}
				</section>
			)}

			<section className="mt-auto rounded-xl border bg-card p-4">
				{dayEndedAt ? (
					<p className="flex items-center gap-2 text-sm font-medium text-primary">
						<CheckCircle2 className="size-5 shrink-0" />
						Journée terminée à{' '}
						{new Intl.DateTimeFormat('fr-FR', {
							hour: '2-digit',
							minute: '2-digit',
						}).format(new Date(dayEndedAt))}
					</p>
				) : (
					<>
						<Label htmlFor="day-end-time" className="text-sm font-semibold">
							Fin de journée
						</Label>
						<p className="mt-1 text-xs text-muted-foreground">
							L'heure que vous indiquez est celle qui compte, même si vous
							remplissez ceci plus tard.
						</p>
						<div className="mt-3 flex gap-2">
							<Input
								id="day-end-time"
								type="time"
								className="h-12 w-28 text-base"
								value={dayEndTime}
								onChange={(event) => onDayEndTimeChange(event.target.value)}
							/>
							<Button
								type="button"
								size="lg"
								className="h-12 flex-1"
								disabled={isEndingDay}
								onClick={onEndDay}
							>
								{isEndingDay ? <Loader2 className="animate-spin" /> : null}
								Terminer ma journée
							</Button>
						</div>
						{running ? (
							<p className="mt-2 text-xs text-amber-600 dark:text-amber-500">
								Le chantier en cours sera clôturé à cette heure.
							</p>
						) : null}
					</>
				)}
			</section>
		</div>
	)
}
