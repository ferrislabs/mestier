import { Check, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { findMember } from '#/pages/hr/lib/rh/absences'
import { formatFull, formatLong, todayIso } from '#/pages/hr/lib/rh/dates'
import {
	describeSpan,
	extraMinutes,
	findSurcharge,
	formatDuration,
} from '#/pages/hr/lib/rh/extra'
import { describeEquivalent, holidayNote } from '#/pages/hr/lib/rh/extra-view'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { ExtraPeriod, RhState } from '#/pages/hr/lib/rh/types'
import { MemberMonthContext } from '#/pages/hr/ui/rh/extra/decision-context'
import { RateBadge } from '#/pages/hr/ui/rh/extra/rate-badge'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'

export function PendingExtras({
	state,
	requests,
	onApprove,
	onReject,
}: {
	state: RhState
	requests: readonly ExtraPeriod[]
	onApprove: (extra: ExtraPeriod) => void
	onReject: (extra: ExtraPeriod) => void
}) {
	return (
		<section aria-labelledby="pending-extras-title" className="mb-8">
			<div className="mb-3 flex items-baseline gap-2">
				<h2 id="pending-extras-title" className="font-semibold text-base">
					À valider
				</h2>
				<span className="text-muted-foreground text-sm tabular">
					{requests.length} déclaration{requests.length > 1 ? 's' : ''}
				</span>
			</div>
			<ul className="grid gap-3">
				{requests.map((extra) => (
					<PendingCard
						key={extra.id}
						state={state}
						extra={extra}
						onApprove={onApprove}
						onReject={onReject}
					/>
				))}
			</ul>
		</section>
	)
}

function PendingCard({
	state,
	extra,
	onApprove,
	onReject,
}: {
	state: RhState
	extra: ExtraPeriod
	onApprove: (extra: ExtraPeriod) => void
	onReject: (extra: ExtraPeriod) => void
}) {
	const member = findMember(state, extra.memberId)
	if (!member) return null
	const name = fullName(member)
	const surcharge = findSurcharge(state, extra.surchargeId)
	const note = holidayNote(extra.date)
	const requestedOn = todayIso(new Date(extra.createdAt))
	return (
		<li className="overflow-hidden rounded-lg border bg-card">
			<div className="grid gap-4 p-4 md:grid-cols-[minmax(0,1fr)_minmax(0,22rem)]">
				<div className="grid min-w-0 content-start gap-3">
					<div className="flex flex-wrap items-center gap-x-3 gap-y-1">
						<MemberAvatar member={member} size="lg" />
						<div className="min-w-0">
							<p className="font-medium leading-tight">{name}</p>
							<p className="text-muted-foreground text-xs">
								Demandé le {formatFull(requestedOn)}
							</p>
						</div>
					</div>
					<div className="grid gap-1.5">
						<p className="font-medium">
							{formatLong(extra.date)}
							{note && (
								<span className="font-normal text-muted-foreground">
									{' '}
									· {note}
								</span>
							)}
						</p>
						<p className="font-medium">
							{describeSpan(extra)}
							<span className="font-normal text-muted-foreground">
								{' '}
								· {formatDuration(extraMinutes(extra))}
							</span>
						</p>
						<p className="flex flex-wrap items-center gap-x-2 gap-y-1 text-[13px]">
							<RateBadge rate={extra.rate} />
							{extra.rate > 0 && (
								<span className="text-muted-foreground">
									{surcharge ? `${surcharge.name} · ` : 'Taux libre · '}
									<span className="text-foreground tabular">
										{describeEquivalent(extra)}
									</span>
								</span>
							)}
						</p>
					</div>
					<p className="border-l-2 pl-3 text-[13px] text-muted-foreground">
						{extra.reason}
					</p>
				</div>
				<aside
					aria-label={`Contexte de décision pour ${name}`}
					className="grid content-start gap-3 rounded-md bg-muted/50 p-3 text-[13px]"
				>
					<p className="font-medium text-[11px] text-muted-foreground uppercase tracking-wide">
						Pour décider
					</p>
					<MemberMonthContext state={state} extra={extra} />
				</aside>
			</div>
			<div className="flex justify-end gap-2 border-t bg-muted/20 px-4 py-2.5">
				<Button
					type="button"
					variant="outline"
					size="sm"
					aria-label={`Refuser la déclaration de ${name}`}
					onClick={() => onReject(extra)}
				>
					<X />
					Refuser
				</Button>
				<Button
					type="button"
					size="sm"
					aria-label={`Valider la déclaration de ${name}`}
					onClick={() => onApprove(extra)}
				>
					<Check />
					Valider
				</Button>
			</div>
		</li>
	)
}
