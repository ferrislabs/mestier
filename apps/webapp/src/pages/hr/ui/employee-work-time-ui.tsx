import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { Employee } from '#/hooks/use-reference-catalog'
import { formatDurationMinutes, type WeeklyGap } from '#/pages/hr/types'
import {
	RhythmSection,
	type RhythmSectionProps,
	WorkSlotsSection,
	type WorkSlotsSectionProps,
} from '#/pages/hr/ui/work-time-editors'
import { MoneyCell, TextField } from '#/pages/settings/ui/primitives'

export interface ContractFormBinding {
	value: string
	isPending: boolean
	error: string | null
	onChange: (value: string) => void
	onSubmit: () => void
}

export interface EmployeeWorkTimeUIProps {
	organizationName: string
	employee: Employee
	weeklyGap: WeeklyGap
	contractForm: ContractFormBinding
	rhythmSection: RhythmSectionProps
	workSlotsSection: WorkSlotsSectionProps
}

export function EmployeeWorkTimeUI({
	organizationName,
	employee,
	weeklyGap,
	contractForm,
	rhythmSection,
	workSlotsSection,
}: EmployeeWorkTimeUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title={employee.name}
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
							Taux horaire
						</span>
						<MoneyCell value={employee.hourly_rate_cents} suffix="/h" />
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

			<WeeklyGapBanner gap={weeklyGap} />

			<RhythmSection {...rhythmSection} />
			<WorkSlotsSection {...workSlotsSection} />
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
