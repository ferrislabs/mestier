import { Plus, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import type { Rhythm } from '#/hooks/use-work-time'
import {
	formatDateFr,
	type RhythmFormValues,
	type RhythmSlotDraft,
	type SlotValidationError,
	WEEKDAYS,
	type WorkSlotDraft,
	type WorkSlotsFormValues,
} from '#/pages/hr/types'

// -- Rythme hebdomadaire ----------------------------------------------------

export interface RhythmSectionProps {
	values: RhythmFormValues
	/** Closed or future versions, shown as read-only history. */
	otherRhythms: Rhythm[]
	openRhythmEffectiveFrom: string | null
	errors: SlotValidationError[]
	isLoading: boolean
	isSaving: boolean
	saveError: string | null
	onEffectiveFromChange: (value: string) => void
	onEffectiveToChange: (value: string) => void
	onSlotChange: (
		key: string,
		patch: Partial<Pick<RhythmSlotDraft, 'startTime' | 'endTime'>>,
	) => void
	onAddSlot: (weekday: number) => void
	onRemoveSlot: (key: string) => void
	onSubmit: () => void
}

export function RhythmSection({
	values,
	otherRhythms,
	openRhythmEffectiveFrom,
	errors,
	isLoading,
	isSaving,
	saveError,
	onEffectiveFromChange,
	onEffectiveToChange,
	onSlotChange,
	onAddSlot,
	onRemoveSlot,
	onSubmit,
}: RhythmSectionProps) {
	const errorByKey = new Map(errors.map((error) => [error.key, error.message]))

	return (
		<SectionCard>
			<SectionHeader
				title="Rythme hebdomadaire"
				description="Le modèle horaire récurrent de l’employé. Une entreprise à horaires réguliers ne remplit que cette section."
			/>
			<div className="flex flex-col gap-5 p-5">
				{isLoading ? (
					<p className="text-sm text-muted-foreground">Chargement du rythme…</p>
				) : (
					<>
						{openRhythmEffectiveFrom ? (
							<p className="text-xs text-muted-foreground">
								Version en cours depuis le{' '}
								{formatDateFr(openRhythmEffectiveFrom)}.
							</p>
						) : (
							<p className="text-xs text-muted-foreground">
								Aucun rythme actif : cet employé ne travaille que sur des plages
								datées, ou n’a pas encore d’horaires renseignés.
							</p>
						)}

						<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
							<DateField
								label="Date de début"
								value={values.effectiveFrom}
								onChange={onEffectiveFromChange}
								error={errorByKey.get('effectiveFrom')}
							/>
							<DateField
								label="Date de fin (optionnel)"
								value={values.effectiveTo}
								onChange={onEffectiveToChange}
								error={errorByKey.get('effectiveTo')}
							/>
						</div>

						<div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
							{WEEKDAYS.map((weekday) => (
								<section
									key={weekday.value}
									aria-label={weekday.label}
									className="flex flex-col gap-2 rounded-lg border p-3"
								>
									<h4 className="text-sm font-semibold">{weekday.label}</h4>
									{values.slots
										.filter((slot) => slot.weekday === weekday.value)
										.map((slot) => (
											<div key={slot.key} className="flex flex-col gap-1">
												<div className="flex items-center gap-1.5">
													<TimeField
														label="Début"
														value={slot.startTime}
														onChange={(startTime) =>
															onSlotChange(slot.key, { startTime })
														}
													/>
													<TimeField
														label="Fin"
														value={slot.endTime}
														onChange={(endTime) =>
															onSlotChange(slot.key, { endTime })
														}
													/>
													<Button
														type="button"
														size="icon-sm"
														variant="ghost"
														onClick={() => onRemoveSlot(slot.key)}
													>
														<Trash2 />
														<span className="sr-only">
															Supprimer le créneau
														</span>
													</Button>
												</div>
												{errorByKey.get(slot.key) ? (
													<p className="text-xs text-destructive">
														{errorByKey.get(slot.key)}
													</p>
												) : null}
											</div>
										))}
									<Button
										type="button"
										size="sm"
										variant="outline"
										onClick={() => onAddSlot(weekday.value)}
									>
										<Plus />
										Ajouter un créneau
									</Button>
								</section>
							))}
						</div>

						{otherRhythms.length > 0 ? (
							<div>
								<h4 className="mb-1 text-xs font-semibold uppercase text-muted-foreground">
									Versions précédentes
								</h4>
								<ul className="flex flex-col gap-1 text-xs text-muted-foreground">
									{otherRhythms.map((rhythm) => (
										<li key={rhythm.id}>
											{formatDateFr(rhythm.effective_from)} →{' '}
											{rhythm.effective_to
												? formatDateFr(rhythm.effective_to)
												: 'en cours'}
										</li>
									))}
								</ul>
							</div>
						) : null}

						{saveError ? (
							<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{saveError}
							</p>
						) : null}

						<div>
							<Button
								type="button"
								onClick={onSubmit}
								disabled={isSaving || errors.length > 0}
							>
								{isSaving ? 'Enregistrement…' : 'Enregistrer le rythme'}
							</Button>
						</div>
					</>
				)}
			</div>
		</SectionCard>
	)
}

// -- Plages de travail --------------------------------------------------

export interface WorkSlotsSectionProps {
	values: WorkSlotsFormValues
	errors: SlotValidationError[]
	isLoading: boolean
	isSaving: boolean
	saveError: string | null
	onFromChange: (value: string) => void
	onToChange: (value: string) => void
	onSlotChange: (
		key: string,
		patch: Partial<Pick<WorkSlotDraft, 'workDate' | 'startTime' | 'endTime'>>,
	) => void
	onAddSlot: () => void
	onRemoveSlot: (key: string) => void
	onSubmit: () => void
}

export function WorkSlotsSection({
	values,
	errors,
	isLoading,
	isSaving,
	saveError,
	onFromChange,
	onToChange,
	onSlotChange,
	onAddSlot,
	onRemoveSlot,
	onSubmit,
}: WorkSlotsSectionProps) {
	const errorByKey = new Map(errors.map((error) => [error.key, error.message]))
	const rangeError = errorByKey.get('range')

	return (
		<SectionCard>
			<SectionHeader
				title="Plages de travail"
				description="Des intervalles posés sur des dates précises. Elles priment sur le rythme ce jour-là — pratique pour les horaires irréguliers."
			/>
			<div className="flex flex-col gap-5 p-5">
				<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<DateField
						label="Début de la période"
						value={values.from}
						onChange={onFromChange}
					/>
					<DateField
						label="Fin de la période"
						value={values.to}
						onChange={onToChange}
					/>
				</div>
				{rangeError ? (
					<p className="text-xs text-destructive">{rangeError}</p>
				) : null}

				{isLoading ? (
					<p className="text-sm text-muted-foreground">
						Chargement des plages…
					</p>
				) : (
					<>
						<div className="flex flex-col gap-2">
							{values.slots.length === 0 ? (
								<p className="text-sm text-muted-foreground">
									Aucune plage sur cette période.
								</p>
							) : (
								values.slots.map((slot, index) => (
									<div key={slot.key} className="flex flex-col gap-1">
										<div className="flex flex-wrap items-center gap-1.5">
											<Input
												aria-label={`Date (plage ${index + 1})`}
												type="date"
												value={slot.workDate}
												onChange={(event) =>
													onSlotChange(slot.key, {
														workDate: event.target.value,
													})
												}
												className="h-8 w-40 text-xs"
											/>
											<Input
												aria-label={`Début (plage ${index + 1})`}
												value={slot.startTime}
												onChange={(event) =>
													onSlotChange(slot.key, {
														startTime: event.target.value,
													})
												}
												className="h-8 w-20 text-xs"
											/>
											<Input
												aria-label={`Fin (plage ${index + 1})`}
												value={slot.endTime}
												onChange={(event) =>
													onSlotChange(slot.key, {
														endTime: event.target.value,
													})
												}
												className="h-8 w-20 text-xs"
											/>
											<Button
												type="button"
												size="icon-sm"
												variant="ghost"
												onClick={() => onRemoveSlot(slot.key)}
											>
												<Trash2 />
												<span className="sr-only">
													Supprimer la plage {index + 1}
												</span>
											</Button>
										</div>
										{errorByKey.get(slot.key) ? (
											<p className="text-xs text-destructive">
												{errorByKey.get(slot.key)}
											</p>
										) : null}
									</div>
								))
							)}
						</div>

						<div>
							<Button
								type="button"
								size="sm"
								variant="outline"
								onClick={onAddSlot}
							>
								<Plus />
								Ajouter une plage
							</Button>
						</div>

						{saveError ? (
							<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{saveError}
							</p>
						) : null}

						<div>
							<Button
								type="button"
								onClick={onSubmit}
								disabled={isSaving || errors.length > 0}
							>
								{isSaving ? 'Enregistrement…' : 'Enregistrer les plages'}
							</Button>
						</div>
					</>
				)}
			</div>
		</SectionCard>
	)
}

// -- Champs partagés ------------------------------------------------------

interface DateFieldProps {
	label: string
	value: string
	onChange: (value: string) => void
	error?: string
}

function DateField({ label, value, onChange, error }: DateFieldProps) {
	return (
		<div className="flex flex-col gap-1.5">
			<span className="text-sm font-medium">{label}</span>
			<Input
				aria-label={label}
				type="date"
				value={value}
				onChange={(event) => onChange(event.target.value)}
			/>
			{error ? <p className="text-xs text-destructive">{error}</p> : null}
		</div>
	)
}

interface TimeFieldProps {
	label: string
	value: string
	onChange: (value: string) => void
}

function TimeField({ label, value, onChange }: TimeFieldProps) {
	return (
		<Input
			aria-label={label}
			placeholder="08:00"
			inputMode="numeric"
			value={value}
			onChange={(event) => onChange(event.target.value)}
			className="h-8 w-20 text-xs"
		/>
	)
}
