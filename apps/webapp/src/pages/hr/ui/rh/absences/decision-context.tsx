import { ArrowRight, Check, TriangleAlert, Users } from 'lucide-react'
import { cn } from '#/lib/utils'
import { findType, type TeamConflict } from '#/pages/hr/lib/rh/absences'
import {
	type BalanceImpact,
	describeDateRange,
	formatDayAmount,
	formatDayCount,
} from '#/pages/hr/lib/rh/absences-view'
import type { RhState } from '#/pages/hr/lib/rh/types'

const MAX_CONFLICTS = 3

export function TeamConflictList({
	state,
	conflicts,
}: {
	state: RhState
	conflicts: readonly TeamConflict[]
}) {
	if (conflicts.length === 0) {
		return (
			<p className="flex items-start gap-2 text-muted-foreground">
				<Check className="mt-0.5 size-3.5 shrink-0 text-state-progress" />
				Personne d’autre n’est absent sur cette période.
			</p>
		)
	}
	const shown = conflicts.slice(0, MAX_CONFLICTS)
	const hidden = conflicts.length - shown.length
	return (
		<ul className="grid gap-1.5">
			{shown.map(({ member, absence }) => (
				<li key={absence.id} className="flex items-start gap-2">
					<Users className="mt-0.5 size-3.5 shrink-0 text-state-today" />
					<span>
						{member.firstName} a aussi une absence{' '}
						{describeDateRange(absence.startDate, absence.endDate)}
						<span className="text-muted-foreground">
							{' '}
							({findType(state, absence.typeId)?.name ?? 'Absence'}
							{absence.status === 'pending' ? ', à valider' : ''})
						</span>
					</span>
				</li>
			))}
			{hidden > 0 && (
				<li className="pl-5.5 text-muted-foreground">
					et {hidden} autre{hidden > 1 ? 's' : ''}
				</li>
			)}
		</ul>
	)
}

export function BalanceImpactLine({ impact }: { impact: BalanceImpact }) {
	const negative = impact.after < 0
	return (
		<div className="grid gap-1.5">
			<p>
				<span className="font-medium">{impact.account.name}</span>
				<span className="text-muted-foreground"> : </span>
				<span className="whitespace-nowrap tabular">
					<span className="text-muted-foreground">disponible </span>
					{formatDayAmount(impact.before)}
					<ArrowRight
						className="mx-1 inline size-3 align-[-1px] text-muted-foreground"
						aria-hidden
					/>
					<span className="sr-only">devient</span>
					<span
						className={cn(
							'font-semibold',
							negative ? 'text-state-alert' : 'text-foreground',
						)}
					>
						{formatDayAmount(impact.after)}
					</span>
				</span>
			</p>
			{negative && (
				<p className="flex items-start gap-2 text-state-alert">
					<TriangleAlert className="mt-0.5 size-3.5 shrink-0" />
					Solde insuffisant : il manque {formatDayCount(impact.shortfall)}
				</p>
			)}
		</div>
	)
}
