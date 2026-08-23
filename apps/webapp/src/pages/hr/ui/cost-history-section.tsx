import { formatMoney, MoneyCell, TextField } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { Switch } from '#/components/ui/switch'
import type {
	CostBasisEntry,
	CostBasisFormValues,
	SlotValidationError,
} from '#/pages/hr/types'
import { formatDateFr, todayIsoDate } from '#/pages/hr/types'
import { DateField } from '#/pages/hr/ui/work-time-editors'

export interface CostHistoryFormBinding {
	values: CostBasisFormValues
	errors: SlotValidationError[]
	isSaving: boolean
	saveError: string | null
	onChange: (patch: Partial<CostBasisFormValues>) => void
	onSubmit: () => void
}

export interface CostHistorySectionProps {
	history: CostBasisEntry[]
	isLoading: boolean
	form: CostHistoryFormBinding
}

export function CostHistorySection({
	history,
	isLoading,
	form,
}: CostHistorySectionProps) {
	const errorByKey = new Map(
		form.errors.map((error) => [error.key, error.message]),
	)
	const isPastDate = Boolean(
		form.values.effectiveFrom && form.values.effectiveFrom < todayIsoDate(),
	)

	return (
		<SectionCard>
			<SectionHeader
				title="Historique du coût"
				description="Chaque changement est daté : un rapport de rentabilité déjà publié garde le chiffre qui était vrai à l’époque."
			/>
			<div className="flex flex-col gap-5 p-5">
				{isLoading ? (
					<p className="text-sm text-muted-foreground">
						Chargement de l’historique…
					</p>
				) : (
					<>
						<div className="grid grid-cols-1 items-end gap-4 sm:grid-cols-4">
							<DateField
								label="Date d’effet"
								value={form.values.effectiveFrom}
								onChange={(effectiveFrom) => form.onChange({ effectiveFrom })}
								error={errorByKey.get('effectiveFrom')}
							/>
							<TextField
								label="Nouveau taux horaire"
								value={form.values.hourlyRate}
								onChange={(hourlyRate) => form.onChange({ hourlyRate })}
								inputMode="decimal"
								suffix="€/h"
								placeholder="Optionnel"
								disabled={form.values.isSalaried}
							/>
							{form.values.isSalaried ? (
								<TextField
									label="Nouveau coût employeur mensuel"
									value={form.values.monthlyCost}
									onChange={(monthlyCost) => form.onChange({ monthlyCost })}
									inputMode="decimal"
									suffix="€/mois"
									placeholder="Salaire chargé"
								/>
							) : (
								<div />
							)}
							<Button
								type="button"
								onClick={form.onSubmit}
								disabled={form.isSaving}
							>
								{form.isSaving ? 'Enregistrement…' : 'Dater ce changement'}
							</Button>
						</div>

						<div className="flex items-center gap-2">
							<Switch
								id="cost-basis-salaried"
								checked={form.values.isSalaried}
								onCheckedChange={(isSalaried) =>
									form.onChange({ isSalaried, hourlyRate: '', monthlyCost: '' })
								}
							/>
							<Label
								htmlFor="cost-basis-salaried"
								className="text-sm font-normal"
							>
								Salarié (coût mensuel plutôt qu’horaire)
							</Label>
						</div>

						{isPastDate ? (
							<p className="text-xs text-muted-foreground">
								Cette date est passée : les rapports de rentabilité qui couvrent
								cette période changeront.
							</p>
						) : null}

						{form.saveError ? (
							<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{form.saveError}
							</p>
						) : null}

						<CostHistoryTable history={history} />
					</>
				)}
			</div>
		</SectionCard>
	)
}

function CostHistoryTable({ history }: { history: CostBasisEntry[] }) {
	if (history.length === 0) {
		return (
			<p className="text-sm text-muted-foreground">
				Aucun historique pour le moment.
			</p>
		)
	}

	// Most recent first: the version somebody just dated is the one they want
	// to see confirmed, not scrolled past.
	const rows = [...history].sort((a, b) =>
		b.effectiveFrom.localeCompare(a.effectiveFrom),
	)

	return (
		<div className="overflow-x-auto">
			<table className="w-full text-sm">
				<thead>
					<tr className="border-b text-left text-xs uppercase text-muted-foreground">
						<th className="py-2 pr-4 font-medium">Période</th>
						<th className="py-2 pr-4 font-medium">Base</th>
						<th className="py-2 font-medium">Coût horaire</th>
					</tr>
				</thead>
				<tbody>
					{rows.map((entry) => (
						<tr key={entry.id} className="border-b last:border-0">
							<td className="py-2 pr-4 tabular-nums">
								{formatDateFr(entry.effectiveFrom)} →{' '}
								{entry.effectiveTo
									? formatDateFr(entry.effectiveTo)
									: 'en cours'}
							</td>
							<td className="py-2 pr-4">
								{entry.isSalaried ? (
									<MoneyCell value={entry.monthlyCostCents} suffix="/mois" />
								) : (
									<MoneyCell value={entry.hourlyRateCents} suffix="/h" />
								)}
							</td>
							<td className="py-2 tabular-nums">
								{entry.effectiveHourlyRateCents === null ? (
									<span className="text-muted-foreground italic">
										Non calculable
									</span>
								) : (
									// The comparable figure, because a monthly amount and an
									// hourly rate cannot be compared by eye — see
									// `salaried_hourly_rate_cents`.
									`${formatMoney(entry.effectiveHourlyRateCents)}/h`
								)}
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	)
}
