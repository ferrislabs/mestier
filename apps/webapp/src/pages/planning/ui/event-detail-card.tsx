import {
	AlignLeft,
	CalendarDays,
	Check,
	CircleSlash,
	MapPin,
	Pencil,
	Tag,
	Trash2,
	User,
	Users,
	X,
} from 'lucide-react'
import type * as React from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import type { AbsenceFormValues } from '#/pages/hr/lib/absences'
import type { EventDetailVM } from '#/pages/planning/lib/build-calendar-model'
import { ABSENCE_LABELS } from '#/pages/planning/lib/entries'
import type { TaskFormValues } from '#/pages/planning/lib/task-form'
import {
	type EventAssigneeOption,
	EventEditForm,
	type EventEditState,
} from '#/pages/planning/ui/event-edit-form'

type TaskStatus = Schemas.TaskStatus

const STATUS_LABELS: Record<TaskStatus, string> = {
	PLANNED: 'Planifiée',
	IN_PROGRESS: 'En cours',
	DONE: 'Terminée',
	CANCELLED: 'Annulée',
}

/** Statuses offered as quick actions, in a task's life-cycle order. */
const QUICK_STATUSES: TaskStatus[] = ['PLANNED', 'IN_PROGRESS', 'DONE']

export interface EventDetailCardProps {
	event: EventDetailVM
	onClose: () => void
	onChangeStatus?: (status: TaskStatus) => void
	onDelete?: () => void
	isPending?: boolean
	/** Draft in progress on this entry, or `null` when reading. */
	editing: EventEditState | null
	assignees: EventAssigneeOption[]
	selectedResourceIds: string[]
	onEdit: () => void
	onEditChange: (patch: Partial<TaskFormValues & AbsenceFormValues>) => void
	onToggleAssignee: (resourceId: string) => void
	onEditSubmit: () => void
	onEditCancel: () => void
	/**
	 * Opens the same full `TaskSheetFeature` the Team and Task-list views use
	 * on this entry — task-only, since an absence has no such sheet (its
	 * editor is `AbsenceFormSheet`, reached from the HR module, not from
	 * here). Absent when the caller has nowhere to send it to, e.g. this
	 * entry has no sheet counterpart.
	 */
	onOpenDetail?: () => void
	quickActionError?: string | null
}

/**
 * Detail panel for an event, opened by clicking its card.
 *
 * It shows what the entry already carries — customer, property, labels,
 * description — rather than sending the user to a screen to read it, and
 * offers in its footer the only actions not worth opening the full sheet for.
 */
export function EventDetailCard({
	event,
	onClose,
	onChangeStatus,
	onDelete,
	isPending,
	editing,
	assignees,
	selectedResourceIds,
	onEdit,
	onEditChange,
	onToggleAssignee,
	onEditSubmit,
	onEditCancel,
	onOpenDetail,
	quickActionError,
}: EventDetailCardProps) {
	const entry = event.entry
	const isTask = entry.kind === 'task'

	return (
		<div className="flex flex-col gap-4">
			<header className="flex items-start justify-between gap-3">
				<p className="min-w-0 truncate text-xs text-muted-foreground">
					{event.dateLabel}
					<span className="mx-1.5">›</span>
					{isTask ? 'Tâche' : 'Absence'}
				</p>
				<div className="flex shrink-0 items-center gap-0.5">
					{editing ? null : (
						<Button
							type="button"
							variant="ghost"
							size="icon-sm"
							aria-label={isTask ? 'Modifier la tâche' : "Modifier l'absence"}
							onClick={onEdit}
						>
							<Pencil />
						</Button>
					)}
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						aria-label="Fermer"
						onClick={onClose}
					>
						<X />
					</Button>
				</div>
			</header>

			{editing ? (
				<>
					<EventEditForm
						state={editing}
						assignees={assignees}
						selectedResourceIds={selectedResourceIds}
						onChange={onEditChange}
						onToggleAssignee={onToggleAssignee}
					/>
					<footer className="flex items-center justify-end gap-2 border-t pt-4">
						<Button
							type="button"
							size="sm"
							variant="ghost"
							disabled={isPending}
							onClick={onEditCancel}
						>
							Annuler
						</Button>
						<Button
							type="button"
							size="sm"
							disabled={isPending || editing.errors.length > 0}
							onClick={onEditSubmit}
						>
							Enregistrer
						</Button>
					</footer>
				</>
			) : (
				<ReadView
					event={event}
					entry={entry}
					isPending={isPending}
					onChangeStatus={onChangeStatus}
					onDelete={onDelete}
					onOpenDetail={isTask ? onOpenDetail : undefined}
				/>
			)}

			{quickActionError ? (
				<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{quickActionError}
				</p>
			) : null}
		</div>
	)
}

interface ReadViewProps {
	event: EventDetailVM
	entry: EventDetailVM['entry']
	isPending?: boolean
	onChangeStatus?: (status: TaskStatus) => void
	onDelete?: () => void
	onOpenDetail?: () => void
}

function ReadView({
	event,
	entry,
	isPending,
	onChangeStatus,
	onDelete,
	onOpenDetail,
}: ReadViewProps) {
	return (
		<>
			<h3 className="text-xl font-medium leading-tight text-foreground">
				{event.title}
			</h3>

			{onOpenDetail ? (
				// The door into the same full task sheet the Team and Task-list
				// views use — this popover only ever offers a status shortcut,
				// never the complete editor (customer, quote, labels, subtasks,
				// comments…), so every field beyond it needs an explicit way out.
				<Button
					type="button"
					variant="link"
					size="sm"
					className="h-auto self-start px-0"
					onClick={onOpenDetail}
				>
					Modifier en détail
				</Button>
			) : null}

			<p className="flex items-center gap-2 text-sm text-foreground">
				<CalendarDays className="size-4 shrink-0 text-muted-foreground" />
				<span className="capitalize">{event.dateLabel}</span>
				<span aria-hidden="true" className="text-muted-foreground">
					•
				</span>
				<span className="tabular-nums">{event.timeLabel}</span>
			</p>

			{event.attendees.length > 0 ? (
				<div className="flex items-center gap-2">
					<span className="flex items-center -space-x-2">
						{event.attendees.slice(0, 4).map((attendee) => (
							<span
								key={attendee.id}
								title={attendee.name}
								className="flex size-8 items-center justify-center rounded-full bg-brand-soft text-xs font-semibold text-primary ring-2 ring-card"
							>
								{attendee.initials}
							</span>
						))}
						{event.attendees.length > 4 ? (
							<span className="flex size-8 items-center justify-center rounded-full bg-muted text-xs font-semibold text-muted-foreground ring-2 ring-card">
								+{event.attendees.length - 4}
							</span>
						) : null}
					</span>
					<span className="truncate text-sm text-muted-foreground">
						{event.attendees.length > 1
							? `${event.attendees.length} assignés`
							: event.attendees[0]?.name}
					</span>
				</div>
			) : (
				<DetailRow icon={Users} muted>
					Personne d’assigné
				</DetailRow>
			)}

			{entry.kind === 'task' ? <TaskRows entry={entry} /> : null}
			{entry.kind === 'absence' ? <AbsenceRows entry={entry} /> : null}

			{entry.kind === 'task' && onChangeStatus ? (
				<footer className="border-t pt-4">
					<p className="mb-2 text-sm font-medium text-foreground">
						Où en est cette tâche ?
					</p>
					<div className="flex flex-wrap gap-2">
						{QUICK_STATUSES.map((status) => {
							const active = entry.status === status
							return (
								<Button
									key={status}
									type="button"
									size="sm"
									variant={active ? 'default' : 'outline'}
									disabled={isPending}
									onClick={() => onChangeStatus(status)}
								>
									{active ? <Check /> : null}
									{STATUS_LABELS[status]}
								</Button>
							)
						})}
						<Button
							type="button"
							size="sm"
							variant="ghost"
							disabled={isPending || entry.status === 'CANCELLED'}
							onClick={() => onChangeStatus('CANCELLED')}
						>
							<CircleSlash />
							Annuler
						</Button>
					</div>
				</footer>
			) : null}

			{entry.kind === 'absence' && onDelete ? (
				<footer className="border-t pt-4">
					<Button
						type="button"
						size="sm"
						variant="ghost"
						className="text-destructive hover:bg-destructive-soft"
						disabled={isPending}
						onClick={onDelete}
					>
						<Trash2 />
						Supprimer cette absence
					</Button>
				</footer>
			) : null}
		</>
	)
}

function TaskRows({
	entry,
}: {
	entry: Extract<EventDetailVM['entry'], { kind: 'task' }>
}) {
	return (
		<>
			{entry.customer_name ? (
				<DetailRow icon={User}>{entry.customer_name}</DetailRow>
			) : null}
			{entry.context_label ? (
				<DetailRow icon={MapPin}>{entry.context_label}</DetailRow>
			) : null}
			{entry.labels.length > 0 ? (
				<DetailRow icon={Tag}>
					<span className="flex flex-wrap gap-1.5">
						{entry.labels.map((label) => (
							<span
								key={label.id}
								className="rounded-full px-2 py-0.5 text-xs font-medium"
								style={{
									backgroundColor: `color-mix(in oklab, ${label.color} 18%, transparent)`,
									color: label.color,
								}}
							>
								{label.name}
							</span>
						))}
					</span>
				</DetailRow>
			) : null}
			{entry.description ? (
				<DetailRow icon={AlignLeft}>
					<span className="whitespace-pre-line">{entry.description}</span>
				</DetailRow>
			) : null}
		</>
	)
}

function AbsenceRows({
	entry,
}: {
	entry: Extract<EventDetailVM['entry'], { kind: 'absence' }>
}) {
	return (
		<>
			<DetailRow icon={Tag}>
				{ABSENCE_LABELS[entry.absence_kind] ?? 'Absence'}
			</DetailRow>
			{entry.note ? (
				<DetailRow icon={AlignLeft}>
					<span className="whitespace-pre-line">{entry.note}</span>
				</DetailRow>
			) : null}
		</>
	)
}

interface DetailRowProps {
	icon: typeof User
	muted?: boolean
	children: React.ReactNode
}

function DetailRow({ icon: Icon, muted, children }: DetailRowProps) {
	return (
		<div className="flex items-start gap-2 text-sm">
			<Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
			<div className={cn('min-w-0', muted && 'text-muted-foreground')}>
				{children}
			</div>
		</div>
	)
}
