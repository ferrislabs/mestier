import { ChevronLeft, ChevronRight } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { formatMonth, startOfMonth } from '#/pages/hr/lib/rh/dates'
import { formatDuration, summarizeExtras } from '#/pages/hr/lib/rh/extra'
import { describeBonus, shiftMonth } from '#/pages/hr/lib/rh/extra-view'
import type { IsoDate, RhState } from '#/pages/hr/lib/rh/types'

function Tile({
	label,
	value,
	hint,
	tone,
}: {
	label: string
	value: string
	hint: string
	tone?: string
}) {
	return (
		<article className="rounded-lg border bg-card px-4 py-3.5">
			<h3 className="text-[13px] text-muted-foreground">{label}</h3>
			<p className={cn('tabular mt-1 font-semibold text-2xl', tone)}>{value}</p>
			<p className="mt-1 text-[13px] text-muted-foreground">{hint}</p>
		</article>
	)
}

function countLabel(count: number, singular: string, plural: string): string {
	return `${count} ${count > 1 ? plural : singular}`
}

export function MonthSummary({
	state,
	month,
	today,
	onMonthChange,
}: {
	state: RhState
	month: IsoDate
	today: IsoDate
	onMonthChange: (month: IsoDate) => void
}) {
	const summary = summarizeExtras(state, month)
	const bonus = summary.equivalentMinutes - summary.minutes
	const current = startOfMonth(today)
	return (
		<section aria-labelledby="month-title" className="mb-8">
			<div className="mb-3 flex flex-wrap items-center justify-between gap-2">
				<h2 id="month-title" className="font-semibold text-base">
					Synthèse du mois
				</h2>
				<div className="flex items-center gap-1">
					{month !== current && (
						<Button
							type="button"
							variant="ghost"
							size="sm"
							className="text-muted-foreground"
							onClick={() => onMonthChange(current)}
						>
							Mois en cours
						</Button>
					)}
					<Button
						type="button"
						variant="outline"
						size="icon-sm"
						aria-label="Mois précédent"
						onClick={() => onMonthChange(shiftMonth(month, -1))}
					>
						<ChevronLeft />
					</Button>
					<output
						aria-live="polite"
						className="min-w-32 text-center font-medium text-sm"
					>
						{formatMonth(month)}
					</output>
					<Button
						type="button"
						variant="outline"
						size="icon-sm"
						aria-label="Mois suivant"
						onClick={() => onMonthChange(shiftMonth(month, 1))}
					>
						<ChevronRight />
					</Button>
				</div>
			</div>
			<div className="grid gap-3 sm:grid-cols-3">
				<Tile
					label="Heures validées"
					value={formatDuration(summary.minutes)}
					hint={
						summary.periods > 0
							? countLabel(
									summary.periods,
									'déclaration validée',
									'déclarations validées',
								)
							: 'Aucune déclaration validée'
					}
				/>
				<Tile
					label="Équivalent payé"
					value={formatDuration(summary.equivalentMinutes)}
					hint={describeBonus(bonus)}
				/>
				<Tile
					label="En attente"
					value={formatDuration(summary.pendingMinutes)}
					hint={
						summary.pendingPeriods > 0
							? countLabel(
									summary.pendingPeriods,
									'déclaration à valider',
									'déclarations à valider',
								)
							: 'Rien à valider'
					}
					tone={summary.pendingPeriods > 0 ? 'text-state-today' : undefined}
				/>
			</div>
		</section>
	)
}
