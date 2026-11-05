import { Link } from '@tanstack/react-router'
import {
	CalendarDays,
	FileText,
	FolderOpen,
	type LucideIcon,
	UserRound,
} from 'lucide-react'
import { RubriqueNav } from '#/components/rubrique-layout'
import {
	contractsSummary,
	documentsSummary,
	identitySummary,
	MEMBER_TABS,
	type MemberTab,
	scheduleSummary,
} from '#/pages/hr/lib/rh/rubriques'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

const ICONS: Record<MemberTab, LucideIcon> = {
	identite: UserRound,
	contrats: FileText,
	documents: FolderOpen,
	'emploi-du-temps': CalendarDays,
}

function summaryOf(
	tab: MemberTab,
	state: RhState,
	member: Member,
	today: IsoDate,
): string {
	switch (tab) {
		case 'identite':
			return identitySummary(member)
		case 'contrats':
			return contractsSummary(member)
		case 'documents':
			return documentsSummary(member.documents.length)
		case 'emploi-du-temps':
			return scheduleSummary(state, member, today)
	}
}

export function RubriqueList({
	state,
	member,
	today,
	selected,
}: {
	state: RhState
	member: Member
	today: IsoDate
	selected: MemberTab
}) {
	const orgPath = useRhOrgPath()
	return (
		<RubriqueNav
			label="Rubriques du membre"
			selected={selected}
			items={MEMBER_TABS.map((tab) => ({
				id: tab.id,
				label: tab.label,
				summary: summaryOf(tab.id, state, member, today),
				icon: ICONS[tab.id],
			}))}
			renderLink={(item, props) => (
				<Link
					to={orgPath('/hr/team/$memberId')}
					params={{ memberId: member.id }}
					search={(previous) => ({ ...previous, tab: item.id })}
					{...props}
				/>
			)}
		/>
	)
}
