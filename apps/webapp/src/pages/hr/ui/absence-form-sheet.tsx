import { CalendarIcon, Loader2, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Calendar } from '#/components/ui/calendar'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import { Textarea } from '#/components/ui/textarea'
import {
	ABSENCE_KIND_LABELS,
	ABSENCE_KINDS,
	type AbsenceFormValues,
	calendarSelectionToRange,
	rangeToCalendarSelection,
} from '#/pages/hr/lib/absences'
import { formatDateFr } from '#/pages/hr/types'

export interface AbsenceMemberOption {
	memberId: string
	displayName: string
}

export interface AbsenceFormSheetProps {
	open: boolean
	mode: 'create' | 'edit'
	values: AbsenceFormValues
	members: AbsenceMemberOption[]
	errors: string[]
	isSaving: boolean
	isDeleting: boolean
	saveError: string | null
	onChange: (patch: Partial<AbsenceFormValues>) => void
	onSubmit: () => void
	onDelete?: () => void
	onOpenChange: (open: boolean) => void
}

/**
 * Create/edit form for an absence — pure presentation, all state lives in
 * the feature (see the planning remodel design doc's "Absences vers le
 * module RH" decision — absence management now lives on the member's
 * work-time screen, `EmployeeWorkTimeFeature`). In edit mode the member is
 * locked rather than offered as a select: the `PATCH /absences/{id}`
 * payload has no `member_id` field, so there is nothing to send even if
 * the user picked a different one.
 */
export function AbsenceFormSheet({
	open,
	mode,
	values,
	members,
	errors,
	isSaving,
	isDeleting,
	saveError,
	onChange,
	onSubmit,
	onDelete,
	onOpenChange,
}: AbsenceFormSheetProps) {
	const memberName =
		members.find((member) => member.memberId === values.memberId)
			?.displayName ?? 'Personne inconnue'
	const canSubmit = errors.length === 0 && !isSaving

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85vh] flex-col gap-0 overflow-hidden sm:max-w-lg">
				<form
					className="flex min-h-0 flex-1 flex-col"
					onSubmit={(event) => {
						event.preventDefault()
						if (canSubmit) onSubmit()
					}}
				>
					<DialogHeader className="border-b pb-4">
						<DialogTitle>
							{mode === 'create' ? 'Nouvelle absence' : 'Modifier l’absence'}
						</DialogTitle>
						<DialogDescription>
							Congé, arrêt ou indisponibilité — l’assignation d’un projet
							pendant cette période n’est jamais bloquée, seulement signalée.
						</DialogDescription>
					</DialogHeader>

					<div className="flex-1 space-y-5 overflow-y-auto py-4">
						<div className="flex flex-col gap-2">
							<Label htmlFor="absence-member">Personne</Label>
							{mode === 'create' ? (
								<Select
									value={values.memberId}
									onValueChange={(memberId) => onChange({ memberId })}
								>
									<SelectTrigger
										id="absence-member"
										aria-label="Personne"
										className="w-full"
									>
										<SelectValue placeholder="Choisir une personne" />
									</SelectTrigger>
									<SelectContent>
										{members.map((member) => (
											<SelectItem key={member.memberId} value={member.memberId}>
												{member.displayName}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							) : (
								<p id="absence-member" className="text-sm font-medium">
									{memberName}
								</p>
							)}
						</div>

						<div className="flex flex-col gap-2">
							<Label htmlFor="absence-kind">Nature</Label>
							<Select
								value={values.kind}
								onValueChange={(kind) =>
									onChange({ kind: kind as AbsenceFormValues['kind'] })
								}
							>
								<SelectTrigger
									id="absence-kind"
									aria-label="Nature"
									className="w-full"
								>
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{ABSENCE_KINDS.map((kind) => (
										<SelectItem key={kind} value={kind}>
											{ABSENCE_KIND_LABELS[kind]}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</div>

						<div className="flex items-center justify-between rounded-lg border p-3">
							<div>
								<p className="text-sm font-medium">Journée entière</p>
								<p className="text-xs text-muted-foreground">
									Sans heure précise, la période couvre chaque jour en entier.
								</p>
							</div>
							<Switch
								checked={values.allDay}
								onCheckedChange={(allDay) => onChange({ allDay })}
								aria-label="Journée entière"
							/>
						</div>

						<RangeField
							value={values.range}
							onChange={(range) => onChange({ range })}
						/>

						{!values.allDay ? (
							<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
								<Input
									aria-label="Heure de début"
									placeholder="08:00"
									value={values.startTime}
									onChange={(event) =>
										onChange({ startTime: event.target.value })
									}
								/>
								<Input
									aria-label="Heure de fin"
									placeholder="18:00"
									value={values.endTime}
									onChange={(event) =>
										onChange({ endTime: event.target.value })
									}
								/>
							</div>
						) : null}

						<div className="flex flex-col gap-2">
							<Label htmlFor="absence-note">Note</Label>
							<Textarea
								id="absence-note"
								value={values.note}
								onChange={(event) => onChange({ note: event.target.value })}
								placeholder="Optionnel"
							/>
						</div>

						{errors.length > 0 ? (
							<ul className="flex flex-col gap-1 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{errors.map((error) => (
									<li key={error}>{error}</li>
								))}
							</ul>
						) : null}

						{saveError ? (
							<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{saveError}
							</p>
						) : null}
					</div>

					<DialogFooter className="border-t pt-4 sm:justify-between">
						{mode === 'edit' && onDelete ? (
							<Button
								type="button"
								variant="ghost"
								className="text-destructive hover:text-destructive"
								disabled={isDeleting}
								onClick={onDelete}
							>
								{isDeleting ? <Loader2 className="animate-spin" /> : <Trash2 />}
								Supprimer
							</Button>
						) : (
							<span />
						)}
						<div className="flex gap-2">
							<Button
								type="button"
								variant="ghost"
								onClick={() => onOpenChange(false)}
							>
								Annuler
							</Button>
							<Button type="submit" disabled={!canSubmit}>
								{isSaving ? <Loader2 className="animate-spin" /> : null}
								{mode === 'create' ? 'Créer l’absence' : 'Enregistrer'}
							</Button>
						</div>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}

/**
 * The single date-range field replacing the old `startDate`/`endDate` pair
 * (see the planning remodel design doc's "champ de plage unique" decision).
 * Fully controlled off `value`/`onChange`, like `planning-toolbar.tsx`'s
 * single-date `Calendar` popover: no local component state, so this stays a
 * pure `ui/` component. `onSelect` fires with a partial range mid-selection
 * (`to` only lands after the second click) — {@link calendarSelectionToRange}
 * folds that into a single-day range rather than needing a "pending click"
 * state of its own.
 */
function RangeField({
	value,
	onChange,
}: {
	value: AbsenceFormValues['range']
	onChange: (range: AbsenceFormValues['range']) => void
}) {
	const label =
		value.from === value.to
			? formatDateFr(value.from)
			: `${formatDateFr(value.from)} – ${formatDateFr(value.to)}`

	return (
		<div className="flex flex-col gap-1.5">
			<Label>Période</Label>
			<Popover>
				<PopoverTrigger asChild>
					<Button
						type="button"
						variant="outline"
						data-testid="absence-range-trigger"
						className="w-full justify-start font-normal"
					>
						<CalendarIcon />
						{label}
					</Button>
				</PopoverTrigger>
				<PopoverContent className="w-auto p-0" align="start">
					<Calendar
						mode="range"
						captionLayout="dropdown"
						selected={rangeToCalendarSelection(value)}
						onSelect={(selection) => {
							const next = calendarSelectionToRange(selection)
							if (next) onChange(next)
						}}
					/>
				</PopoverContent>
			</Popover>
		</div>
	)
}
