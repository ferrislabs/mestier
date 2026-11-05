import { CalendarDays, Clock, Zap } from 'lucide-react'
import { formatLong } from '#/pages/hr/lib/rh/dates'
import { formatDuration, formatRate } from '#/pages/hr/lib/rh/extra'
import {
	bonusMinutes,
	describeBonus,
	holidayNote,
	netMinutes,
	paidMinutes,
} from '#/pages/hr/lib/rh/extra-view'
import type { ExtraDraft } from '#/pages/hr/lib/rh/types'

export function ExtraRecap({
	draft,
	approve,
}: {
	draft: ExtraDraft
	approve: boolean
}) {
	const net = netMinutes(draft)
	const paid = paidMinutes(draft)
	const bonus = bonusMinutes(draft)
	const note = holidayNote(draft.date)
	const rateKnown = Number.isFinite(draft.rate)
	return (
		<aside
			aria-label="Récapitulatif de la déclaration"
			className="grid content-start gap-4 self-start rounded-lg bg-muted/50 p-4 text-[13px]"
		>
			<p className="font-medium text-[11px] text-muted-foreground uppercase tracking-wide">
				Récapitulatif
			</p>
			<dl className="grid gap-3">
				<div>
					<dt className="text-muted-foreground">Durée nette</dt>
					<dd className="tabular font-semibold text-lg">
						{net > 0 ? formatDuration(net) : '—'}
					</dd>
				</div>
				<div>
					<dt className="text-muted-foreground">Équivalent payé</dt>
					<dd className="tabular font-semibold text-lg">
						{paid > 0 ? formatDuration(paid) : '—'}
					</dd>
					<dd className="text-muted-foreground">
						{!rateKnown
							? 'Renseigne le taux de majoration.'
							: draft.rate > 0
								? `${formatRate(draft.rate)}${net > 0 ? `, ${describeBonus(bonus)}` : ''}`
								: 'Sans majoration'}
					</dd>
				</div>
			</dl>
			<div className="grid gap-2 border-t pt-3">
				{draft.date !== '' && (
					<p className="flex items-start gap-2">
						<CalendarDays className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
						<span>
							{formatLong(draft.date)}
							{note && (
								<span className="font-medium text-state-today"> · {note}</span>
							)}
						</span>
					</p>
				)}
				<p className="flex items-start gap-2">
					{approve ? (
						<Zap className="mt-0.5 size-3.5 shrink-0 text-state-progress" />
					) : (
						<Clock className="mt-0.5 size-3.5 shrink-0 text-state-today" />
					)}
					{approve
						? 'Validée immédiatement'
						: 'Cette déclaration sera envoyée à validation'}
				</p>
			</div>
		</aside>
	)
}
