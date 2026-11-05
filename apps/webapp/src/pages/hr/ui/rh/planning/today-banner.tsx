import { formatLong } from '#/pages/hr/lib/rh/dates'
import { fullName } from '#/pages/hr/lib/rh/members'
import { type AbsentEntry, presenceOn } from '#/pages/hr/lib/rh/planning'
import {
	presenceSummary,
	type TypeIndex,
	typeInfo,
} from '#/pages/hr/lib/rh/planning-model'
import type { IsoDate, RhState } from '#/pages/hr/lib/rh/types'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import { TypeChip } from '#/pages/hr/ui/rh/shared/type-chip'

function suffixOf(entry: AbsentEntry): string | undefined {
	const parts = [
		entry.part === 'morning'
			? 'matin'
			: entry.part === 'afternoon'
				? 'après-midi'
				: null,
		entry.pending ? 'à valider' : null,
	].filter(Boolean)
	return parts.length > 0 ? parts.join(' · ') : undefined
}

export function TodayBanner({
	state,
	types,
	today,
}: {
	state: RhState
	types: TypeIndex
	today: IsoDate
}) {
	const presence = presenceOn(state, today)
	return (
		<section
			aria-label="Aujourd’hui"
			className="flex shrink-0 flex-wrap items-center gap-x-5 gap-y-2 rounded-lg border bg-card px-4 py-2.5"
		>
			<div className="flex items-baseline gap-2">
				<h2 className="font-semibold text-sm">Aujourd’hui</h2>
				<span className="text-muted-foreground text-xs">
					{formatLong(today)}
				</span>
			</div>
			<p className="font-medium text-sm tabular-nums">
				{presenceSummary(presence)}
			</p>
			<span aria-hidden className="hidden h-5 w-px bg-border sm:block" />
			{presence.absent.length === 0 ? (
				<p className="text-muted-foreground text-sm">Personne n’est absent.</p>
			) : (
				<ul
					aria-label="Absents aujourd’hui"
					className="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-1.5"
				>
					{presence.absent.map((entry) => (
						<li key={entry.member.id} className="flex items-center gap-2">
							<MemberAvatar member={entry.member} size="sm" />
							<span className="text-sm">{fullName(entry.member)}</span>
							<TypeChip
								type={typeInfo(types, entry.typeId)}
								suffix={suffixOf(entry)}
							/>
						</li>
					))}
				</ul>
			)}
		</section>
	)
}
