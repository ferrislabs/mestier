import { Link } from '@tanstack/react-router'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'

export type ProjectTab = 'overview' | 'doe'

interface ProjectTabsProps {
	organizationSlug: string
	projectId: string
	active: ProjectTab
	doeLabel: string
}

const TAB_CLASS =
	'-mb-px border-b-2 px-3 py-2 text-sm font-medium transition-colors'

/**
 * The project's own screens: its figures, then its DOE. The DOE belongs to
 * the project, so it is reached from here rather than from a list of its own.
 */
export function ProjectTabs({
	organizationSlug,
	projectId,
	active,
	doeLabel,
}: ProjectTabsProps) {
	const tab = (current: ProjectTab) =>
		cn(
			TAB_CLASS,
			active === current
				? 'border-primary text-foreground'
				: 'border-transparent text-muted-foreground hover:text-foreground',
		)

	return (
		<nav aria-label="Écrans du projet" className="flex gap-1 border-b">
			<Link
				to={buildOrgPath(
					organizationSlug,
					'/planification/projects/$projectId',
				)}
				params={{ projectId }}
				aria-current={active === 'overview' ? 'page' : undefined}
				className={tab('overview')}
			>
				Aperçu
			</Link>
			<Link
				to={buildOrgPath(
					organizationSlug,
					'/planification/projects/$projectId/doe',
				)}
				params={{ projectId }}
				aria-current={active === 'doe' ? 'page' : undefined}
				className={tab('doe')}
			>
				{doeLabel}
			</Link>
		</nav>
	)
}
