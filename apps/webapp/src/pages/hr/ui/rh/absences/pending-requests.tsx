import { Check, Paperclip, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	absenceDays,
	findMember,
	findType,
	teamConflicts,
} from '#/pages/hr/lib/rh/absences'
import {
	balanceImpact,
	describeSpan,
	formatDayCount,
	partsNote,
} from '#/pages/hr/lib/rh/absences-view'
import { formatFull, todayIso } from '#/pages/hr/lib/rh/dates'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { Absence, RhState } from '#/pages/hr/lib/rh/types'
import {
	BalanceImpactLine,
	TeamConflictList,
} from '#/pages/hr/ui/rh/absences/decision-context'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import { TypeChip } from '#/pages/hr/ui/rh/shared/type-chip'

export function PendingRequests({
	state,
	requests,
	onApprove,
	onReject,
}: {
	state: RhState
	requests: readonly Absence[]
	onApprove: (absence: Absence) => void
	onReject: (absence: Absence) => void
}) {
	const today = todayIso()
	return (
		<section aria-labelledby="pending-title" className="mb-8">
			<div className="mb-3 flex items-baseline gap-2">
				<h2 id="pending-title" className="font-semibold text-base">
					À valider
				</h2>
				<span className="text-muted-foreground text-sm tabular">
					{requests.length} demande{requests.length > 1 ? 's' : ''}
				</span>
			</div>
			<ul className="grid gap-3">
				{requests.map((absence) => (
					<PendingCard
						key={absence.id}
						state={state}
						absence={absence}
						today={today}
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
	absence,
	today,
	onApprove,
	onReject,
}: {
	state: RhState
	absence: Absence
	today: string
	onApprove: (absence: Absence) => void
	onReject: (absence: Absence) => void
}) {
	const member = findMember(state, absence.memberId)
	const type = findType(state, absence.typeId)
	if (!member || !type) return null
	const name = fullName(member)
	const days = absenceDays(absence, member, type)
	const note = partsNote(absence)
	const impact = balanceImpact(state, absence, today, true)
	const requestedOn = todayIso(new Date(absence.createdAt))
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
						<TypeChip type={type} className="ml-auto md:ml-2" />
					</div>
					<div>
						<p className="font-medium">
							{describeSpan(absence)}
							<span className="font-normal text-muted-foreground">
								{' '}
								· {formatDayCount(days)}
							</span>
						</p>
						{note && <p className="text-muted-foreground text-xs">{note}</p>}
					</div>
					{absence.comment !== '' && (
						<p className="border-l-2 pl-3 text-[13px] text-muted-foreground">
							{absence.comment}
						</p>
					)}
					{absence.proofName && (
						<p className="flex items-center gap-1.5 text-[13px]">
							<Paperclip className="size-3.5 text-muted-foreground" />
							{absence.proofName}
						</p>
					)}
				</div>
				<aside
					aria-label={`Contexte de décision pour ${name}`}
					className="grid content-start gap-3 rounded-md bg-muted/50 p-3 text-[13px]"
				>
					<p className="font-medium text-[11px] text-muted-foreground uppercase tracking-wide">
						Pour décider
					</p>
					<TeamConflictList
						state={state}
						conflicts={teamConflicts(state, absence)}
					/>
					{impact ? (
						<BalanceImpactLine impact={impact} />
					) : (
						<p className="text-muted-foreground">
							Ce motif ne débite aucun compte de congés.
						</p>
					)}
				</aside>
			</div>
			<div className="flex justify-end gap-2 border-t bg-muted/20 px-4 py-2.5">
				<Button
					type="button"
					variant="outline"
					size="sm"
					aria-label={`Refuser la demande de ${name}`}
					onClick={() => onReject(absence)}
				>
					<X />
					Refuser
				</Button>
				<Button
					type="button"
					size="sm"
					aria-label={`Valider la demande de ${name}`}
					onClick={() => onApprove(absence)}
				>
					<Check />
					Valider
				</Button>
			</div>
		</li>
	)
}
