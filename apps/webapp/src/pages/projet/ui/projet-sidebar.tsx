import { Link } from '@tanstack/react-router'
import {
	FileText,
	Layers,
	LayoutTemplate,
	type LucideIcon,
	Settings2,
	SquareKanban,
	Target,
} from 'lucide-react'
import { buildOrgPath } from '#/modules/org-path'
import { ENTRY } from '#/pages/projet/lib/project-path'
import {
	SidebarEntryContent,
	SidebarSection,
	sidebarItemClassName,
} from '#/pages/projet/ui/sidebar-parts'

export interface SelectedProject {
	id: string
	missionCount: number
	doeNote: string
}

export interface ProjetSidebarProps {
	organizationSlug: string
	active: ReadonlySet<string>
	projectsCount: number
	modelsCount: number
	selected: SelectedProject | null
	isCollapsed: (groupId: string) => boolean
	onToggleGroup: (groupId: string) => void
}

interface EntryProps {
	organizationSlug: string
	to: string
	active: boolean
	icon: LucideIcon
	label: string
	count?: number
	note?: string
}

function Entry({
	organizationSlug,
	to,
	active,
	icon: Icon,
	label,
	count,
	note,
}: EntryProps) {
	return (
		<li>
			<Link
				to={buildOrgPath(organizationSlug, to)}
				activeOptions={{ exact: true }}
				aria-current={active ? 'page' : undefined}
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

export function ProjetSidebar({
	organizationSlug,
	active,
	projectsCount,
	modelsCount,
	selected,
	isCollapsed,
	onToggleGroup,
}: ProjetSidebarProps) {
	return (
		<nav aria-label="Navigation Projet" className="flex flex-col gap-4">
			<ul className="flex flex-col gap-px">
				<Entry
					organizationSlug={organizationSlug}
					to="/projet"
					active={active.has(ENTRY.projects)}
					icon={Layers}
					label="Tous les projets"
					count={projectsCount}
				/>
				<Entry
					organizationSlug={organizationSlug}
					to="/projet/doe-models"
					active={active.has(ENTRY.models)}
					icon={LayoutTemplate}
					label="Modèles de DOE"
					count={modelsCount}
				/>
			</ul>
			{selected && (
				<SidebarSection
					title="Projet sélectionné"
					collapsed={isCollapsed('selected')}
					onToggle={() => onToggleGroup('selected')}
				>
					<Entry
						organizationSlug={organizationSlug}
						to={`/projet/${selected.id}`}
						active={active.has(ENTRY.overview)}
						icon={SquareKanban}
						label="Aperçu"
					/>
					<Entry
						organizationSlug={organizationSlug}
						to={`/projet/${selected.id}/missions`}
						active={active.has(ENTRY.missions)}
						icon={Target}
						label="Missions"
						count={selected.missionCount}
					/>
					<Entry
						organizationSlug={organizationSlug}
						to={`/projet/${selected.id}/doe`}
						active={active.has(ENTRY.doe)}
						icon={FileText}
						label="DOE"
						note={selected.doeNote}
					/>
					<Entry
						organizationSlug={organizationSlug}
						to={`/projet/${selected.id}/settings`}
						active={active.has(ENTRY.settings)}
						icon={Settings2}
						label="Réglages"
					/>
				</SidebarSection>
			)}
		</nav>
	)
}
