import { formatMoney, MoneyCell, TextField } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { Member } from '#/hooks/use-reference-catalog'
import {
	formatDateFr,
	formatDurationMinutes,
	type WeeklyGap,
} from '#/pages/hr/types'
import {
	AbsenceFormSheet,
	type AbsenceFormSheetProps,
} from '#/pages/hr/ui/absence-form-sheet'
import {
	AbsencesSection,
	type AbsencesSectionProps,
} from '#/pages/hr/ui/absences-section'
import {
	CostHistorySection,
	type CostHistorySectionProps,
} from '#/pages/hr/ui/cost-history-section'
import {
	RhythmSection,
	type RhythmSectionProps,
	WorkSlotsSection,
	type WorkSlotsSectionProps,
} from '#/pages/hr/ui/work-time-editors'

export interface ContractFormBinding {
	value: string
	isPending: boolean
	error: string | null
	onChange: (value: string) => void
	onSubmit: () => void
}

export interface EmployeeWorkTimeUIProps {
	organizationName: string
	member: Member
	/** `null` when the member has no employee profile yet. */
	hourlyRateCents: number | null
	/** Costed from a monthly amount rather than by the hour. */
	isSalaried: boolean
	monthlyCostCents: number | null
	/**
	 * The hourly figure profitability actually uses, computed server-side. `null`
	 * when it cannot be stated — which, on this page, means the field just below
	 * is the one to fill.
	 */
	effectiveHourlyRateCents: number | null
	/** The date the currently open cost basis version took effect, `null` when there is none yet. */
	openCostBasisEffectiveFrom: string | null
	weeklyGap: WeeklyGap
	contractForm: ContractFormBinding
	costHistorySection: CostHistorySectionProps
	rhythmSection: RhythmSectionProps
	workSlotsSection: WorkSlotsSectionProps
	absencesSection: AbsencesSectionProps
	/** The create/edit absence form — see the planning remodel design doc: absence management moved here from the planning module. */
	absenceSheet: AbsenceFormSheetProps
}

export function EmployeeWorkTimeUI({
	organizationName,
	member,
	hourlyRateCents,
	isSalaried,
	monthlyCostCents,
	effectiveHourlyRateCents,
	openCostBasisEffectiveFrom,
	weeklyGap,
	contractForm,
	costHistorySection,
	rhythmSection,
	workSlotsSection,
	absencesSection,
	absenceSheet,
}: EmployeeWorkTimeUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title={member.display_name}
				description="Base contractuelle, rythme hebdomadaire et plages de travail."
			/>

			<SectionCard>
				<SectionHeader
					title="Base contractuelle et taux horaire"
					description="La base contractuelle n’est pas dérivée des plages planifiées : l’écart est une information, pas une anomalie à corriger."
				/>
				<div className="grid grid-cols-1 items-end gap-4 p-5 sm:grid-cols-3">
					<div className="flex flex-col gap-1.5">
						<span className="text-sm font-medium text-muted-foreground">
							{isSalaried ? 'Coût employeur' : 'Taux horaire'}
						</span>
						{isSalaried ? (
							// Without this branch the page said "Taux horaire : Non
							// renseigné" for a salaried person — the same misreading the
							// team list had, on the very page they are sent to in order to
							// fix it.
							<div className="flex flex-col">
								<MoneyCell value={monthlyCostCents} suffix="/mois" />
								{effectiveHourlyRateCents === null ? null : (
									<span className="text-xs text-muted-foreground">
										soit {formatMoney(effectiveHourlyRateCents)}/h
									</span>
								)}
							</div>
						) : (
							<MoneyCell value={hourlyRateCents} suffix="/h" />
						)}
						{openCostBasisEffectiveFrom ? (
							<span className="text-xs text-muted-foreground">
								En vigueur depuis le {formatDateFr(openCostBasisEffectiveFrom)}
							</span>
						) : null}
					</div>
					<TextField
						label="Base contractuelle"
						value={contractForm.value}
						onChange={contractForm.onChange}
						suffix="/sem."
					/>
					<Button
						type="button"
						onClick={contractForm.onSubmit}
						disabled={contractForm.isPending}
					>
						Enregistrer la base contractuelle
					</Button>
				</div>
				{contractForm.error ? (
					<p className="px-5 pb-4 text-sm text-destructive">
						{contractForm.error}
					</p>
				) : null}
			</SectionCard>

			<CostHistorySection {...costHistorySection} />

			<WeeklyGapBanner gap={weeklyGap} />

			<RhythmSection {...rhythmSection} />
			<WorkSlotsSection {...workSlotsSection} />
			<AbsencesSection {...absencesSection} />
			<AbsenceFormSheet {...absenceSheet} />
		</PageShell>
	)
}

EmployeeWorkTimeUI.Loading = function EmployeeWorkTimeLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				Chargement du temps de travail…
			</SectionCard>
		</PageShell>
	)
}

function WeeklyGapBanner({ gap }: { gap: WeeklyGap }) {
	const tone =
		gap.deltaMinutes === 0
			? 'neutral'
			: gap.deltaMinutes > 0
				? 'brand'
				: 'warning'

	return (
		<SectionCard className="flex flex-col gap-3 p-5 sm:flex-row sm:items-center sm:justify-between">
			<div className="flex flex-wrap items-center gap-6">
				<Metric
					label="Planifiées / semaine"
					value={formatDurationMinutes(gap.plannedMinutes)}
				/>
				<Metric
					label="Base contractuelle"
					value={formatDurationMinutes(gap.contractMinutes)}
				/>
			</div>
			<StatusBadge tone={tone}>
				Écart : {formatSignedDelta(gap.deltaMinutes)}
			</StatusBadge>
		</SectionCard>
	)
}

function Metric({ label, value }: { label: string; value: string }) {
	return (
		<div>
			<p className="text-xs uppercase text-muted-foreground">{label}</p>
			<p className="text-lg font-semibold tabular-nums">{value}</p>
		</div>
	)
}

function formatSignedDelta(minutes: number): string {
	return minutes >= 0
		? `+${formatDurationMinutes(minutes)}`
		: formatDurationMinutes(minutes)
}
