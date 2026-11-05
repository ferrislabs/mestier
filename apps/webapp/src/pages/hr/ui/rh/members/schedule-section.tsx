import { Link } from '@tanstack/react-router'
import { Clock } from 'lucide-react'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { AbsencesCard } from '#/pages/hr/ui/rh/members/absences-card'
import { ExtraCard } from '#/pages/hr/ui/rh/members/extra-card'
import { MemberCalendar } from '#/pages/hr/ui/rh/members/member-calendar'
import { RhythmCard } from '#/pages/hr/ui/rh/members/rhythm-card'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

export function ScheduleSection({
	state,
	member,
	today,
	month,
	onMonthChange,
}: {
	state: RhState
	member: Member
	today: IsoDate
	month: IsoDate
	onMonthChange: (month: IsoDate | null) => void
}) {
	const orgPath = useRhOrgPath()
	return (
		<RubriqueLayout
			wide
			title="Emploi du temps"
			description="Calendrier, semaine type, absences et heures extra."
			actions={
				<Button variant="outline" size="sm" asChild>
					<Link
						to={orgPath('/hr/team/$memberId/work-time')}
						params={{ memberId: member.id }}
					>
						<Clock aria-hidden /> Temps de travail
					</Link>
				</Button>
			}
		>
			<MemberCalendar
				state={state}
				member={member}
				today={today}
				month={month}
				onMonthChange={onMonthChange}
			/>
			<RhythmCard state={state} member={member} today={today} month={month} />
			<AbsencesCard state={state} member={member} today={today} />
			<ExtraCard state={state} member={member} today={today} />
		</RubriqueLayout>
	)
}
