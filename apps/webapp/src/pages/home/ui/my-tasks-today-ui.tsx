import {
	AlertCircle,
	Clock,
	Loader2,
	Play,
	RefreshCw,
	Square,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { FieldTask, TimeEntry } from '#/hooks/use-field'
import { elapsedLabel, taskWindowLabel } from '#/pages/field/types'

interface MyTasksTodayUIProps {
	tasks: FieldTask[]
	isLoading: boolean
	loadFailed: boolean
	onRetry: () => void
	running: TimeEntry | null
	runningTaskTitle: string | null
	/** Re-rendered every minute so the running clock advances. */
	now: number
	pendingTaskId: string | null
	isStopping: boolean
	error: string | null
	onStart: (taskId: string) => void
	onStop: () => void
	/** The task whose "declare a time" form is open, at most one at a time. */
	declaringTaskId: string | null
	declareStart: string
	declareEnd: string
	isDeclaring: boolean
	onToggleDeclare: (taskId: string | null) => void
	onDeclareStartChange: (value: string) => void
	onDeclareEndChange: (value: string) => void
	onDeclareSubmit: () => void
}

/**
 * Lets the logged-in employee clock in and out of their own tasks straight
 * from the main dashboard, without detouring through the phone-oriented
 * `/field` screen. Reuses the same field endpoints — every action here is
 * already scoped server-side to the caller's own identity, so no backend
 * change was needed to bring this into the regular panel.
 *
 * Deliberately a subset of the field screen: no photo capture, no day-end, no
 * forgotten-entry recovery. Those stay specific to the on-site app; this card
 * only covers "start/stop work on a task", which is what "valider ses tâches"
 * from the desk needs.
 */
export function MyTasksTodayUI({
	tasks,
	isLoading,
	loadFailed,
	onRetry,
	running,
	runningTaskTitle,
	now,
	pendingTaskId,
	isStopping,
	error,
	onStart,
	onStop,
	declaringTaskId,
	declareStart,
	declareEnd,
	isDeclaring,
	onToggleDeclare,
	onDeclareStartChange,
	onDeclareEndChange,
	onDeclareSubmit,
}: MyTasksTodayUIProps) {
	return (
		<SectionCard>
			<SectionHeader
				title="Mes chantiers du jour"
				description="Démarrez ou clôturez votre pointage, ou déclarez un temps déjà travaillé si vous avez oublié de lancer le compteur."
			/>
			<div className="space-y-3 p-5">
				{error ? (
					<div className="flex items-center gap-2 rounded-md border border-destructive/30 bg-destructive-soft px-3 py-2 text-sm text-destructive">
						<AlertCircle className="size-4 shrink-0" />
						{error}
					</div>
				) : null}

				{running ? (
					<div className="flex items-center justify-between gap-3 rounded-md border border-primary/30 bg-brand-soft px-3 py-2">
						<div className="min-w-0">
							<p className="truncate text-sm font-semibold">
								{runningTaskTitle ?? 'Chantier'}
							</p>
							<p className="text-xs text-muted-foreground">
								En cours depuis {elapsedLabel(running.started_at, now)}
							</p>
						</div>
						<Button
							type="button"
							size="sm"
							variant="outline"
							disabled={isStopping}
							onClick={onStop}
						>
							{isStopping ? <Loader2 className="animate-spin" /> : <Square />}
							Clôturer
						</Button>
					</div>
				) : null}

				{loadFailed ? (
					<div className="flex flex-col items-center gap-3 rounded-md border border-dashed p-4 text-center text-sm text-muted-foreground">
						Impossible de charger vos chantiers du jour.
						<Button type="button" size="sm" variant="outline" onClick={onRetry}>
							<RefreshCw />
							Réessayer
						</Button>
					</div>
				) : isLoading ? (
					<div className="flex items-center justify-center gap-2 p-6 text-sm text-muted-foreground">
						<Loader2 className="size-4 animate-spin" />
						Chargement…
					</div>
				) : tasks.length === 0 ? (
					<p className="rounded-md border border-dashed p-4 text-center text-sm text-muted-foreground">
						Aucun chantier prévu aujourd'hui.
					</p>
				) : (
					<ul className="divide-y">
						{tasks.map((task) => {
							const isCurrent = running?.task_id === task.id
							const windowLabel = taskWindowLabel(task)
							const isDeclaringThis = declaringTaskId === task.id

							return (
								<li key={task.id} className="flex flex-col gap-2 py-3">
									<div className="flex items-center justify-between gap-3">
										<div className="min-w-0">
											<div className="flex items-center gap-2">
												<p className="truncate text-sm font-medium">
													{task.title}
												</p>
												{isCurrent ? (
													<StatusBadge tone="brand">En cours</StatusBadge>
												) : null}
											</div>
											{windowLabel ? (
												<p className="text-xs text-muted-foreground">
													{windowLabel}
												</p>
											) : null}
										</div>
										<div className="flex shrink-0 items-center gap-1">
											{isCurrent ? null : (
												<>
													<Button
														type="button"
														size="sm"
														variant="ghost"
														onClick={() =>
															onToggleDeclare(isDeclaringThis ? null : task.id)
														}
													>
														<Clock />
														Déclarer un temps
													</Button>
													<Button
														type="button"
														size="sm"
														variant="outline"
														disabled={pendingTaskId === task.id}
														onClick={() => onStart(task.id)}
													>
														{pendingTaskId === task.id ? (
															<Loader2 className="animate-spin" />
														) : (
															<Play />
														)}
														{running ? 'Basculer' : 'Démarrer'}
													</Button>
												</>
											)}
										</div>
									</div>

									{isDeclaringThis ? (
										<div className="flex flex-wrap items-end gap-2 rounded-md border bg-muted/40 p-3">
											<div className="flex flex-col gap-1">
												<Label
													htmlFor={`declare-start-${task.id}`}
													className="text-xs text-muted-foreground"
												>
													Début
												</Label>
												<Input
													id={`declare-start-${task.id}`}
													type="time"
													className="h-9 w-28"
													value={declareStart}
													onChange={(event) =>
														onDeclareStartChange(event.target.value)
													}
												/>
											</div>
											<div className="flex flex-col gap-1">
												<Label
													htmlFor={`declare-end-${task.id}`}
													className="text-xs text-muted-foreground"
												>
													Fin
												</Label>
												<Input
													id={`declare-end-${task.id}`}
													type="time"
													className="h-9 w-28"
													value={declareEnd}
													onChange={(event) =>
														onDeclareEndChange(event.target.value)
													}
												/>
											</div>
											<Button
												type="button"
												size="sm"
												disabled={isDeclaring}
												onClick={onDeclareSubmit}
											>
												{isDeclaring ? (
													<Loader2 className="animate-spin" />
												) : null}
												Valider
											</Button>
											<Button
												type="button"
												size="sm"
												variant="ghost"
												disabled={isDeclaring}
												onClick={() => onToggleDeclare(null)}
											>
												Annuler
											</Button>
											<p className="w-full text-xs text-muted-foreground">
												Pour aujourd'hui uniquement. Ce temps sera marqué comme
												déclaré après coup, pas mesuré en direct.
											</p>
										</div>
									) : null}
								</li>
							)
						})}
					</ul>
				)}
			</div>
		</SectionCard>
	)
}
