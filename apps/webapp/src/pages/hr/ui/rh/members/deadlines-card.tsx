import { Link } from '@tanstack/react-router'
import { CalendarClock } from 'lucide-react'
import { buildOrgPath } from '#/modules/org-path'
import { formatDate } from '#/pages/hr/lib/rh/format'
import {
	DEADLINE_LABELS,
	findMemberById,
	fullName,
	upcomingDeadlines,
} from '#/pages/hr/lib/rh/members'
import type { IsoDate, RhState } from '#/pages/hr/lib/rh/types'
import { DeadlineNote } from '#/pages/hr/ui/rh/members/deadline-note'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'

const HORIZON_DAYS = 60

export function DeadlinesCard({
	state,
	today,
	organizationSlug,
}: {
	state: RhState
	today: IsoDate
	organizationSlug: string
}) {
	const deadlines = upcomingDeadlines(state, today, HORIZON_DAYS)
	if (deadlines.length === 0) return null
	return (
		<section
			aria-labelledby="rh-deadlines-title"
			className="mb-4 rounded-lg border bg-card"
		>
			<header className="flex items-center gap-2 border-b px-4 py-2.5">
				<CalendarClock className="size-4 text-muted-foreground" aria-hidden />
				<h2 id="rh-deadlines-title" className="font-medium text-sm">
					Échéances à venir
				</h2>
				<span className="text-[12px] text-muted-foreground">
					dans les {HORIZON_DAYS} prochains jours
				</span>
			</header>
			<ul className="divide-y">
				{deadlines.map((deadline) => {
					const member = findMemberById(state, deadline.memberId)
					if (!member) return null
					return (
						<li key={`${deadline.memberId}-${deadline.kind}`}>
							<Link
								to={buildOrgPath(organizationSlug, '/hr/team/$memberId')}
								params={{ memberId: member.id }}
								className="flex items-center gap-3 px-4 py-2 outline-none transition-colors hover:bg-accent/60 focus-visible:bg-accent"
							>
								<MemberAvatar member={member} size="md" />
								<span className="min-w-0 flex-1 truncate text-[13.5px]">
									<span className="font-medium">
										{DEADLINE_LABELS[deadline.kind]}
									</span>
									<span className="text-muted-foreground">
										{' · '}
										{fullName(member)}
									</span>
								</span>
								<span className="tabular text-[13px] text-muted-foreground">
									{formatDate(deadline.date)}
								</span>
								<DeadlineNote
									daysLeft={deadline.daysLeft}
									className="w-24 text-right"
								/>
							</Link>
						</li>
					)
				})}
			</ul>
		</section>
	)
}
