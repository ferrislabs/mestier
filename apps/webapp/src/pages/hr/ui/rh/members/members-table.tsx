import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { StatusBadge } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { cn } from '#/lib/utils'
import { currentContract } from '#/pages/hr/lib/rh/contracts'
import { contractCountLabel, contractTerm } from '#/pages/hr/lib/rh/format'
import { findMemberById, fullName } from '#/pages/hr/lib/rh/members'
import { columnSummary } from '#/pages/hr/lib/rh/schedule-mode'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import type { AccessState } from '#/pages/hr/types'
import {
	ArchivedBadge,
	MemberContractBadge,
} from '#/pages/hr/ui/rh/members/contract-badge'
import {
	type MemberRealInfo,
	NO_REAL_INFO,
} from '#/pages/hr/ui/rh/members/member-access'
import {
	ContractBaseCell,
	RateCell,
} from '#/pages/hr/ui/rh/members/member-cost-cells'
import { MemberRowActions } from '#/pages/hr/ui/rh/members/member-row-actions'
import { TodayCell } from '#/pages/hr/ui/rh/members/today-cell'
import { WeekDots } from '#/pages/hr/ui/rh/members/week-dots'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

const ACCESS_LABEL: Record<AccessState, string> = {
	none: 'Aucun accès',
	invited: 'Invité',
	linkedAccount: 'Compte lié',
}

const ACCESS_TONE: Record<AccessState, 'neutral' | 'warning' | 'success'> = {
	none: 'neutral',
	invited: 'warning',
	linkedAccount: 'success',
}

export function MembersTable({
	state,
	members,
	today,
	realInfo,
	hrDataRestricted,
	renderRoles,
	onEdit,
	onInvite,
	onDelete,
}: {
	state: RhState
	members: readonly Member[]
	today: IsoDate
	realInfo: ReadonlyMap<string, MemberRealInfo>
	hrDataRestricted: boolean
	renderRoles: (memberId: string) => ReactNode
	onEdit: (member: Member) => void
	onInvite: (member: Member) => void
	onDelete: (member: Member) => void
}) {
	const orgPath = useRhOrgPath()
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Membre</TableHead>
						<TableHead>Contrat</TableHead>
						<TableHead className="max-xl:hidden">Responsable</TableHead>
						<TableHead className="max-lg:hidden">Semaine type</TableHead>
						<TableHead>Aujourd’hui</TableHead>
						<TableHead>Accès</TableHead>
						<TableHead>Rôles</TableHead>
						<TableHead>Taux</TableHead>
						<TableHead className="max-xl:hidden">Base contractuelle</TableHead>
						<TableHead className="w-12 pr-4">
							<span className="sr-only">Actions</span>
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{members.map((member) => {
						const info = realInfo.get(member.id) ?? NO_REAL_INFO
						const manager = findMemberById(state, member.managerId)
						const archived = member.status === 'archived'
						const contract = currentContract(member, today)
						const rhythm = columnSummary(contract.schedule, today)
						return (
							<TableRow
								key={member.id}
								className={cn(
									'group relative cursor-pointer focus-within:bg-muted/50',
									archived && 'opacity-60',
								)}
							>
								<TableCell className="py-2.5 pl-4">
									<div className="flex items-center gap-3">
										<MemberAvatar member={member} size="lg" />
										<span className="min-w-0">
											<Link
												to={orgPath('/rh/membres/$memberId')}
												params={{ memberId: member.id }}
												className="flex items-center gap-2 font-medium outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
											>
												<span className="truncate">{fullName(member)}</span>
												{archived && <ArchivedBadge />}
											</Link>
											<span className="block truncate text-[12px] text-muted-foreground">
												{member.jobTitle || 'Poste non renseigné'}
											</span>
										</span>
									</div>
								</TableCell>
								<TableCell>
									<span className="flex items-center gap-1.5">
										<MemberContractBadge member={member} today={today} />
										{member.contracts.length > 1 && (
											<span className="text-[12px] text-muted-foreground">
												{contractCountLabel(member.contracts.length)}
											</span>
										)}
									</span>
									<span className="mt-0.5 block text-[12px] text-muted-foreground">
										{contractTerm(contract, today)}
									</span>
								</TableCell>
								<TableCell className="text-muted-foreground max-xl:hidden">
									{manager ? fullName(manager) : '—'}
								</TableCell>
								<TableCell className="max-lg:hidden">
									<span className="tabular mb-1 flex items-baseline gap-1.5 text-[13px]">
										{rhythm.hours}
										{rhythm.tag && (
											<span className="font-medium text-[10px] text-muted-foreground uppercase tracking-wide">
												{rhythm.tag}
											</span>
										)}
									</span>
									{rhythm.week ? (
										<WeekDots schedule={rhythm.week} />
									) : (
										<span className="block text-[12px] text-muted-foreground">
											Jour par jour
										</span>
									)}
								</TableCell>
								<TableCell>
									<TodayCell state={state} member={member} today={today} />
								</TableCell>
								<TableCell>
									<StatusBadge tone={ACCESS_TONE[info.access]}>
										{ACCESS_LABEL[info.access]}
									</StatusBadge>
								</TableCell>
								<TableCell className="relative z-10">
									{renderRoles(member.id)}
								</TableCell>
								<TableCell>
									<RateCell info={info} memberId={member.id} />
								</TableCell>
								<TableCell className="max-xl:hidden">
									<ContractBaseCell
										info={info}
										hrDataRestricted={hrDataRestricted}
									/>
								</TableCell>
								<TableCell className="relative z-10 pr-4">
									<MemberRowActions
										memberId={member.id}
										memberName={fullName(member)}
										access={info.access}
										archived={archived}
										onEdit={() => onEdit(member)}
										onInvite={() => onInvite(member)}
										onDelete={() => onDelete(member)}
									/>
								</TableCell>
							</TableRow>
						)
					})}
				</TableBody>
			</Table>
		</div>
	)
}
