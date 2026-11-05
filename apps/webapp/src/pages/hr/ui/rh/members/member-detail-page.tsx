import { Link } from '@tanstack/react-router'
import { ArrowLeft, UserRoundX } from 'lucide-react'
import { Button } from '#/components/ui/button'
import type { MemberTab } from '#/pages/hr/lib/rh/rubriques'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { ContractsSection } from '#/pages/hr/ui/rh/members/contracts-section'
import { DocumentsSection } from '#/pages/hr/ui/rh/members/documents-section'
import { IdentitySection } from '#/pages/hr/ui/rh/members/identity-section'
import {
	BackLink,
	MemberSidebar,
} from '#/pages/hr/ui/rh/members/member-sidebar'
import { ScheduleSection } from '#/pages/hr/ui/rh/members/schedule-section'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

export interface MemberDetailPageProps {
	state: RhState
	member: Member | undefined
	today: IsoDate
	tab: MemberTab
	month: IsoDate
	onMonthChange: (month: IsoDate | null) => void
	onRename: (names: { firstName: string; lastName: string }) => void
}

export function MemberDetailPage({
	state,
	member,
	today,
	tab,
	month,
	onMonthChange,
	onRename,
}: MemberDetailPageProps) {
	const orgPath = useRhOrgPath()

	if (!member)
		return (
			<div className="min-h-0 flex-1 overflow-y-auto">
				<div className="mx-auto w-full max-w-3xl px-6 py-8">
					<BackLink />
					<div className="mt-4 rounded-lg border border-dashed bg-card px-6 py-12 text-center">
						<UserRoundX
							className="mx-auto mb-3 size-8 text-muted-foreground"
							aria-hidden
						/>
						<p className="font-medium">Ce membre est introuvable.</p>
						<p className="mt-1 text-[13px] text-muted-foreground">
							Il n’existe pas ou l’adresse n’est plus valide.
						</p>
						<Button variant="outline" size="sm" className="mt-4" asChild>
							<Link to={orgPath('/hr/team')}>
								<ArrowLeft aria-hidden /> Retour aux membres
							</Link>
						</Button>
					</div>
				</div>
			</div>
		)

	return (
		<div className="min-h-0 flex-1 overflow-y-auto lg:grid lg:grid-cols-[minmax(20rem,24rem)_minmax(0,1fr)] lg:grid-rows-[minmax(0,1fr)] lg:overflow-hidden">
			<MemberSidebar state={state} member={member} today={today} tab={tab} />
			<section aria-label="Détail du membre" className="min-w-0 lg:min-h-0">
				{tab === 'identite' && (
					<IdentitySection
						state={state}
						member={member}
						today={today}
						onRename={onRename}
					/>
				)}
				{tab === 'contrats' && (
					<ContractsSection state={state} member={member} today={today} />
				)}
				{tab === 'documents' && <DocumentsSection member={member} />}
				{tab === 'emploi-du-temps' && (
					<ScheduleSection
						state={state}
						member={member}
						today={today}
						month={month}
						onMonthChange={onMonthChange}
					/>
				)}
			</section>
		</div>
	)
}
