import { Loader2, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { Switch } from '#/components/ui/switch'
import { Textarea } from '#/components/ui/textarea'
import {
	ABSENCE_KIND_LABELS,
	ABSENCE_KINDS,
	type AbsenceFormValues,
} from '#/pages/planning/lib/absences'

export interface AbsenceEmployeeOption {
	employeeId: string
	displayName: string
}

export interface AbsenceFormSheetProps {
	open: boolean
	mode: 'create' | 'edit'
	values: AbsenceFormValues
	employees: AbsenceEmployeeOption[]
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
 * the feature (see the planning design doc's CRUD-des-absences scope). In
 * edit mode the employee is locked rather than offered as a select: the
 * `PATCH /absences/{id}` payload has no `employee_id` field, so there is
 * nothing to send even if the user picked a different one.
 */
export function AbsenceFormSheet({
	open,
	mode,
	values,
	employees,
	errors,
	isSaving,
	isDeleting,
	saveError,
	onChange,
	onSubmit,
	onDelete,
	onOpenChange,
}: AbsenceFormSheetProps) {
	const employeeName =
		employees.find((employee) => employee.employeeId === values.employeeId)
			?.displayName ?? 'Employé inconnu'
	const canSubmit = errors.length === 0 && !isSaving

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-lg">
				<form
					className="flex min-h-0 flex-1 flex-col"
					onSubmit={(event) => {
						event.preventDefault()
						if (canSubmit) onSubmit()
					}}
				>
					<SheetHeader className="border-b">
						<SheetTitle>
							{mode === 'create' ? 'Nouvelle absence' : 'Modifier l’absence'}
						</SheetTitle>
						<SheetDescription>
							Congé, arrêt ou indisponibilité — l’assignation d’un chantier
							pendant cette période n’est jamais bloquée, seulement signalée.
						</SheetDescription>
					</SheetHeader>

					<div className="flex-1 space-y-5 overflow-y-auto p-4">
						<div className="flex flex-col gap-2">
							<Label htmlFor="absence-employee">Employé</Label>
							{mode === 'create' ? (
								<Select
									value={values.employeeId}
									onValueChange={(employeeId) => onChange({ employeeId })}
								>
									<SelectTrigger
										id="absence-employee"
										aria-label="Employé"
										className="w-full"
									>
										<SelectValue placeholder="Choisir un employé" />
									</SelectTrigger>
									<SelectContent>
										{employees.map((employee) => (
											<SelectItem
												key={employee.employeeId}
												value={employee.employeeId}
											>
												{employee.displayName}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							) : (
								<p id="absence-employee" className="text-sm font-medium">
									{employeeName}
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

						<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
							<DateField
								label="Date de début"
								value={values.startDate}
								onChange={(startDate) => onChange({ startDate })}
							/>
							<DateField
								label="Date de fin"
								value={values.endDate}
								onChange={(endDate) => onChange({ endDate })}
							/>
						</div>

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

					<SheetFooter className="border-t bg-background sm:flex-row sm:justify-between">
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
					</SheetFooter>
				</form>
			</SheetContent>
		</Sheet>
	)
}

function DateField({
	label,
	value,
	onChange,
}: {
	label: string
	value: string
	onChange: (value: string) => void
}) {
	return (
		<div className="flex flex-col gap-1.5">
			<Label>{label}</Label>
			<Input
				aria-label={label}
				type="date"
				value={value}
				onChange={(event) => onChange(event.target.value)}
			/>
		</div>
	)
}
