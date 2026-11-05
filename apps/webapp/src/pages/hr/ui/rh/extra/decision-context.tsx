import { CalendarCheck, Check, Hourglass } from 'lucide-react'
import { formatMonth } from '#/pages/hr/lib/rh/dates'
import { formatDuration, summarizeExtras } from '#/pages/hr/lib/rh/extra'
import { describeBonus } from '#/pages/hr/lib/rh/extra-view'
import type { ExtraPeriod, RhState } from '#/pages/hr/lib/rh/types'

export function MemberMonthContext({
	state,
	extra,
}: {
	state: RhState
	extra: ExtraPeriod
}) {
	const summary = summarizeExtras(state, extra.date, extra.memberId)
	const month = formatMonth(extra.date)
	const others = summary.pendingPeriods - (extra.status === 'pending' ? 1 : 0)
	return (
		<div className="grid gap-2">
			{summary.periods === 0 ? (
				<p className="flex items-start gap-2 text-muted-foreground">
					<Check className="mt-0.5 size-3.5 shrink-0 text-state-progress" />
					Aucune heure extra validée en {month}.
				</p>
			) : (
				<p className="flex items-start gap-2">
					<CalendarCheck className="mt-0.5 size-3.5 shrink-0 text-state-progress" />
					<span>
						Déjà validé en {month} :{' '}
						<span className="font-semibold tabular">
							{formatDuration(summary.minutes)}
						</span>
						<span className="text-muted-foreground">
							{' '}
							({summary.periods} déclaration{summary.periods > 1 ? 's' : ''},{' '}
							{formatDuration(summary.equivalentMinutes)} payées
							{summary.equivalentMinutes > summary.minutes
								? `, ${describeBonus(summary.equivalentMinutes - summary.minutes)}`
								: ''}
							)
						</span>
					</span>
				</p>
			)}
			{others > 0 && (
				<p className="flex items-start gap-2 text-muted-foreground">
					<Hourglass className="mt-0.5 size-3.5 shrink-0 text-state-today" />
					{others} autre{others > 1 ? 's' : ''} déclaration
					{others > 1 ? 's' : ''} à valider ce mois-là.
				</p>
			)}
		</div>
	)
}
