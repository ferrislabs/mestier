import { Link } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { formatMonth, formatShort } from '#/pages/hr/lib/rh/dates'
import {
	describeSpan,
	extraMinutes,
	formatDuration,
	formatRate,
	summarizeExtras,
} from '#/pages/hr/lib/rh/extra'
import { recentExtras, totalLabel } from '#/pages/hr/lib/rh/extra-summary'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { DetailCard, EmptyNote } from '#/pages/hr/ui/rh/members/detail-card'
import { AbsenceStatusBadge } from '#/pages/hr/ui/rh/shared/absence-status-badge'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

function Figure({
	label,
	value,
	tone,
}: {
	label: string
	value: string
	tone?: string
}) {
	return (
		<div className="grid gap-0.5">
			<dt className="text-[12px] text-muted-foreground">{label}</dt>
			<dd className={cn('tabular font-semibold text-lg leading-tight', tone)}>
				{value}
			</dd>
		</div>
	)
}

export function ExtraCard({
	state,
	member,
	today,
}: {
	state: RhState
	member: Member
	today: IsoDate
}) {
	const orgPath = useRhOrgPath()
	const summary = summarizeExtras(state, today, member.id)
	const recent = recentExtras(state, member.id)
	const archived = member.status === 'archived'
	const declare = !archived && (
		<Button variant="outline" size="sm" asChild>
			<Link to={orgPath('/hr/extra/new')} search={{ member: member.id }}>
				<Plus aria-hidden /> Déclarer des heures
			</Link>
		</Button>
	)
	return (
		<DetailCard
			title="Heures extra"
			action={
				recent.length > 0 && (
					<Link
						to={orgPath('/hr/extra')}
						search={{ member: member.id }}
						className="rounded-sm text-[13px] text-primary underline-offset-4 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
					>
						Voir tout
					</Link>
				)
			}
		>
			{recent.length === 0 ? (
				<div className="grid justify-items-start gap-3">
					<EmptyNote>Aucune heure extra déclarée.</EmptyNote>
					{declare}
				</div>
			) : (
				<>
					<p className="mb-2 text-[12px] text-muted-foreground capitalize">
						{formatMonth(today)}
					</p>
					<dl className="grid grid-cols-3 gap-3">
						<Figure label="Validées" value={totalLabel(summary.minutes)} />
						<Figure
							label="Équivalent payé"
							value={totalLabel(summary.equivalentMinutes)}
						/>
						<Figure
							label="En attente"
							value={totalLabel(summary.pendingMinutes)}
							tone={summary.pendingMinutes > 0 ? 'text-state-today' : undefined}
						/>
					</dl>
					<ul className="mt-4 divide-y border-t">
						{recent.map((extra) => (
							<li
								key={extra.id}
								className="flex items-center gap-3 py-2 text-[13px]"
							>
								<span className="min-w-0 flex-1">
									<span className="block">
										<span className="tabular">{formatShort(extra.date)}</span>
										<span className="tabular ml-2 text-muted-foreground">
											{describeSpan(extra)}
										</span>
									</span>
									<span className="block text-[12px] text-muted-foreground">
										{formatDuration(extraMinutes(extra))} ·{' '}
										{formatRate(extra.rate)}
									</span>
								</span>
								<AbsenceStatusBadge status={extra.status} />
							</li>
						))}
					</ul>
					{declare && <div className="mt-3 border-t pt-3">{declare}</div>}
				</>
			)}
		</DetailCard>
	)
}
