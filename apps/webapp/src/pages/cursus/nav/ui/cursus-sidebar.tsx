import { Link } from '@tanstack/react-router'
import {
	BookOpen,
	BookUser,
	Building2,
	Layers,
	type LucideIcon,
	School,
	UserRound,
	UserRoundCog,
	Users,
} from 'lucide-react'
import type { ReactNode } from 'react'
import { buildOrgPath } from '#/modules/org-path'
import {
	CURSUS_ENTRY,
	type CursusEntry,
} from '#/pages/cursus/nav/lib/cursus-path'
import {
	SidebarEntryContent,
	SidebarSection,
	sidebarItemClassName,
} from '#/pages/projet/ui/sidebar-parts'

export interface CursusSidebarCounts {
	promotions?: number
	triads?: number
	apprentices?: number
	tutors?: number
	companies?: number
	trainings?: number
	establishments?: number
}

export interface CursusSidebarProps {
	organizationSlug: string
	active: CursusEntry | null
	counts: CursusSidebarCounts
	canView: boolean
	isParticipant: boolean
	isCollapsed: (groupId: string) => boolean
	onToggleGroup: (groupId: string) => void
}

type Audience = 'staff' | 'participant'

interface EntryDef {
	key: CursusEntry
	to: string
	icon: LucideIcon
	label: string
	count?: keyof CursusSidebarCounts
	audience: Audience
}

interface GroupDef {
	id: string
	title: string | null
	entries: readonly EntryDef[]
}

const GROUPS: readonly GroupDef[] = [
	{
		id: 'main',
		title: null,
		entries: [
			{
				key: CURSUS_ENTRY.promotions,
				to: '/cursus/promotions',
				icon: Layers,
				label: 'Promotions',
				count: 'promotions',
				audience: 'staff',
			},
			{
				key: CURSUS_ENTRY.triads,
				to: '/cursus/trinomes',
				icon: Users,
				label: 'Trinômes',
				count: 'triads',
				audience: 'staff',
			},
			{
				key: CURSUS_ENTRY.myTriads,
				to: '/cursus/mes-trinomes',
				icon: BookUser,
				label: 'Mes trinômes',
				audience: 'participant',
			},
		],
	},
	{
		id: 'directory',
		title: 'Annuaire',
		entries: [
			{
				key: CURSUS_ENTRY.apprentices,
				to: '/cursus/apprentis',
				icon: UserRound,
				label: 'Apprentis',
				count: 'apprentices',
				audience: 'staff',
			},
			{
				key: CURSUS_ENTRY.tutors,
				to: '/cursus/tuteurs',
				icon: UserRoundCog,
				label: 'Tuteurs',
				count: 'tutors',
				audience: 'staff',
			},
			{
				key: CURSUS_ENTRY.companies,
				to: '/cursus/entreprises',
				icon: Building2,
				label: 'Entreprises',
				count: 'companies',
				audience: 'staff',
			},
		],
	},
	{
		id: 'reference',
		title: 'Référentiel',
		entries: [
			{
				key: CURSUS_ENTRY.trainings,
				to: '/cursus/formations',
				icon: BookOpen,
				label: 'Formations',
				count: 'trainings',
				audience: 'staff',
			},
			{
				key: CURSUS_ENTRY.establishments,
				to: '/cursus/etablissements',
				icon: School,
				label: 'Établissements',
				count: 'establishments',
				audience: 'staff',
			},
		],
	},
]

function Entry({
	organizationSlug,
	entry,
	active,
	count,
}: {
	organizationSlug: string
	entry: EntryDef
	active: boolean
	count?: number
}) {
	const Icon = entry.icon
	return (
		<li>
			<Link
				to={buildOrgPath(organizationSlug, entry.to)}
				aria-current={active ? 'page' : undefined}
				className={sidebarItemClassName(active)}
			>
				<SidebarEntryContent
					leading={<Icon className="size-4" strokeWidth={1.8} aria-hidden />}
					label={entry.label}
					count={count}
				/>
			</Link>
		</li>
	)
}

export function CursusSidebar({
	organizationSlug,
	active,
	counts,
	canView,
	isParticipant,
	isCollapsed,
	onToggleGroup,
}: CursusSidebarProps) {
	const visible = (audience: Audience) =>
		audience === 'staff' ? canView : isParticipant
	return (
		<nav aria-label="Navigation Cursus" className="flex flex-col gap-4">
			{GROUPS.map((group) => {
				const entries = group.entries.filter((entry) => visible(entry.audience))
				if (entries.length === 0) return null
				const items: ReactNode = entries.map((entry) => (
					<Entry
						key={entry.key}
						organizationSlug={organizationSlug}
						entry={entry}
						active={active === entry.key}
						count={entry.count ? counts[entry.count] : undefined}
					/>
				))
				if (group.title === null) {
					return (
						<ul key={group.id} className="flex flex-col gap-px">
							{items}
						</ul>
					)
				}
				return (
					<SidebarSection
						key={group.id}
						title={group.title}
						collapsed={isCollapsed(group.id)}
						onToggle={() => onToggleGroup(group.id)}
					>
						{items}
					</SidebarSection>
				)
			})}
		</nav>
	)
}
