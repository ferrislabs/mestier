import { Link } from '@tanstack/react-router'
import {
	AlarmClockPlus,
	BadgePercent,
	CalendarCheck,
	CalendarX,
	Clock,
	Hourglass,
	IdCard,
	Inbox,
	type LucideIcon,
	Package,
	Sun,
	Tag,
	Timer,
	UserRound,
	Users,
} from 'lucide-react'
import type { PermissionName } from '#/hooks/use-permissions'
import { buildOrgPath } from '#/modules/org-path'
import type { DeadlineKind } from '#/pages/hr/lib/rh/members'
import {
	SidebarEntryContent,
	SidebarSection,
	sidebarItemClassName,
} from '#/pages/projet/ui/sidebar-parts'
import { ENTRY } from '#/pages/rh-nav/lib/sidebar-active'
import type { SidebarCounts } from '#/pages/rh-nav/lib/sidebar-counts'
import type { SidebarDeadline } from '#/pages/rh-nav/lib/sidebar-deadlines'

export interface RhSidebarAccount {
	id: string
	name: string
	acquisition: { kind: 'monthly' | 'annual' }
}

export interface RhSidebarProps {
	organizationSlug: string
	active: ReadonlySet<string>
	counts: SidebarCounts | null
	currentMemberId: string
	accounts: readonly RhSidebarAccount[]
	deadlines: readonly SidebarDeadline[]
	visible: (permission: PermissionName | null) => boolean
	isCollapsed: (groupId: string) => boolean
	onToggleGroup: (groupId: string) => void
}

const ACCOUNT_ICONS: Record<'monthly' | 'annual', LucideIcon> = {
	monthly: Sun,
	annual: Timer,
}

const DEADLINE_ICONS: Record<DeadlineKind, LucideIcon> = {
	trial: Clock,
	contract_end: CalendarX,
	work_permit: IdCard,
}

interface EntryProps {
	organizationSlug: string
	to: string
	search?: Readonly<Record<string, string>>
	active: boolean
	icon: LucideIcon
	label: string
	count?: number
	note?: string
	title?: string
}

function Entry({
	organizationSlug,
	to,
	search,
	active,
	icon: Icon,
	label,
	count,
	note,
	title,
}: EntryProps) {
	return (
		<li>
			<Link
				to={buildOrgPath(organizationSlug, to)}
				search={search}
				activeOptions={{ exact: true }}
				aria-current={active ? 'page' : undefined}
				title={title}
				className={sidebarItemClassName(active)}
			>
				<SidebarEntryContent
					leading={<Icon className="size-4" strokeWidth={1.8} aria-hidden />}
					label={label}
					count={count}
					note={note}
				/>
			</Link>
		</li>
	)
}

export function RhSidebar({
	organizationSlug,
	active,
	counts,
	currentMemberId,
	accounts,
	deadlines,
	visible,
	isCollapsed,
	onToggleGroup,
}: RhSidebarProps) {
	const entry = (props: Omit<EntryProps, 'organizationSlug'>) => (
		<Entry key={props.label} organizationSlug={organizationSlug} {...props} />
	)
	const canManageReference = visible('MANAGE_REFERENCE')
	const canManagePlanning = visible('MANAGE_PLANNING')

	return (
		<nav aria-label="Navigation RH" className="flex flex-col gap-4">
			<ul className="flex flex-col gap-px">
				{visible('MANAGE_MEMBERS') &&
					entry({
						to: '/rh/membres',
						active: active.has(ENTRY.members),
						icon: Users,
						label: 'Tous les membres',
						count: counts?.active,
					})}
				{canManagePlanning &&
					entry({
						to: '/rh/planning',
						active: active.has(ENTRY.planning),
						icon: CalendarCheck,
						label: 'Planning',
					})}
				{canManagePlanning &&
					entry({
						to: '/rh/temps-de-travail',
						active: active.has(ENTRY.workTime),
						icon: Clock,
						label: 'Temps de travail',
					})}
				{canManageReference &&
					entry({
						to: '/rh/materiel',
						active: active.has(ENTRY.equipment),
						icon: Package,
						label: 'Matériel',
					})}
			</ul>
			{canManageReference && (
				<SidebarSection
					title="Absences"
					collapsed={isCollapsed('absences')}
					onToggle={() => onToggleGroup('absences')}
				>
					{[
						entry({
							to: '/rh/absences',
							search: { statut: 'pending' },
							active: active.has(ENTRY.pending),
							icon: Hourglass,
							label: 'À valider',
							count: counts?.pending,
						}),
						entry({
							to: '/rh/absences',
							active: active.has(ENTRY.requests),
							icon: Inbox,
							label: 'Toutes les demandes',
							count: counts?.requests,
						}),
						entry({
							to: '/rh/absences',
							search: { membre: currentMemberId },
							active: active.has(ENTRY.mine),
							icon: UserRound,
							label: 'Mes demandes',
							count: counts?.mine,
						}),
						entry({
							to: '/rh/absences',
							search: { onglet: 'motifs' },
							active: active.has(ENTRY.types),
							icon: Tag,
							label: 'Motifs',
							count: counts?.types,
						}),
					]}
				</SidebarSection>
			)}
			{canManagePlanning && (
				<SidebarSection
					title="Heures extra"
					collapsed={isCollapsed('extra')}
					onToggle={() => onToggleGroup('extra')}
				>
					{[
						entry({
							to: '/rh/extra',
							search: { statut: 'pending' },
							active: active.has(ENTRY.extraPending),
							icon: Hourglass,
							label: 'À valider',
							count: counts?.extraPending,
						}),
						entry({
							to: '/rh/extra',
							active: active.has(ENTRY.extraRequests),
							icon: AlarmClockPlus,
							label: 'Toutes les déclarations',
							count: counts?.extraRequests,
						}),
						entry({
							to: '/rh/extra',
							search: { membre: currentMemberId },
							active: active.has(ENTRY.extraMine),
							icon: UserRound,
							label: 'Mes déclarations',
							count: counts?.extraMine,
						}),
						entry({
							to: '/rh/extra',
							search: { onglet: 'majorations' },
							active: active.has(ENTRY.extraSurcharges),
							icon: BadgePercent,
							label: 'Majorations',
							count: counts?.surcharges,
						}),
					]}
				</SidebarSection>
			)}
			{canManageReference && accounts.length > 0 && (
				<SidebarSection
					title="Droits"
					collapsed={isCollapsed('rights')}
					onToggle={() => onToggleGroup('rights')}
				>
					{accounts.map((account, index) =>
						entry({
							to: '/rh/droits',
							search: index === 0 ? undefined : { compte: account.id },
							active: active.has(ENTRY.account(account.id)),
							icon: ACCOUNT_ICONS[account.acquisition.kind],
							label: account.name,
						}),
					)}
				</SidebarSection>
			)}
			{visible('MANAGE_MEMBERS') && deadlines.length > 0 && (
				<SidebarSection
					title="Échéances"
					collapsed={isCollapsed('deadlines')}
					onToggle={() => onToggleGroup('deadlines')}
				>
					{deadlines.map((deadline) =>
						entry({
							to: `/rh/membres/${deadline.memberId}`,
							active: false,
							icon: DEADLINE_ICONS[deadline.kind],
							label: deadline.label,
							note: deadline.note,
							title: deadline.title,
						}),
					)}
				</SidebarSection>
			)}
		</nav>
	)
}
