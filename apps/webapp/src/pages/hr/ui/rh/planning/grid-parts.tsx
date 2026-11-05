import { fullName } from '#/pages/hr/lib/rh/members'
import type {
	AbsenceInitial,
	ExtraInitial,
	TypeIndex,
} from '#/pages/hr/lib/rh/planning-model'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'

export type GridProps = {
	state: RhState
	types: TypeIndex
	members: readonly Member[]
	team: readonly Member[]
	days: readonly IsoDate[]
	today: IsoDate
	query: string
	onRequestAbsence: (initial: AbsenceInitial) => void
	onRequestExtra: (initial: ExtraInitial) => void
}

export function MemberLabel({
	member,
	compact = false,
}: {
	member: Member
	compact?: boolean
}) {
	return (
		<span className="flex min-w-0 items-center gap-2.5">
			<MemberAvatar member={member} size={compact ? 'sm' : 'md'} />
			<span className="min-w-0">
				<span className="block truncate font-medium text-sm leading-tight">
					{fullName(member)}
				</span>
				<span className="block truncate text-muted-foreground text-xs leading-tight">
					{member.jobTitle}
				</span>
			</span>
		</span>
	)
}

export function EmptyRow({
	colSpan,
	query,
}: {
	colSpan: number
	query: string
}) {
	return (
		<tr>
			<td
				colSpan={colSpan}
				className="px-4 py-10 text-center text-muted-foreground text-sm"
			>
				Aucun membre ne correspond à « {query.trim()} ».
			</td>
		</tr>
	)
}
