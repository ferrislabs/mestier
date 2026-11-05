import { Link } from '@tanstack/react-router'
import { BreadcrumbSeparator } from '#/modules/breadcrumb-parts'
import { buildOrgPath } from '#/modules/org-path'
import type { ProjectSection } from '#/pages/projet/lib/project-path'
import {
	ProjectSwitcher,
	type ProjectSwitcherProps,
} from '#/pages/projet/ui/project-switcher'

export const SECTION_LABELS: Record<ProjectSection, string> = {
	overview: 'Aperçu',
	missions: 'Missions',
	doe: 'DOE',
	settings: 'Réglages',
}

export interface ProjetCrumbsProps extends ProjectSwitcherProps {
	organizationSlug: string
	section: ProjectSection
	onPreview: boolean
}

export function ProjetCrumbs({
	organizationSlug,
	section,
	onPreview,
	...switcher
}: ProjetCrumbsProps) {
	return (
		<>
			<BreadcrumbSeparator />
			<ProjectSwitcher {...switcher} />
			<BreadcrumbSeparator />
			{onPreview ? (
				<>
					<Link
						to={buildOrgPath(organizationSlug, '/projet/$projectId/doe')}
						params={{ projectId: switcher.currentId }}
						className="truncate rounded-sm text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
					>
						DOE
					</Link>
					<BreadcrumbSeparator />
					<span aria-current="page" className="truncate font-medium">
						Aperçu du PDF
					</span>
				</>
			) : (
				<span aria-current="page" className="truncate font-medium">
					{SECTION_LABELS[section]}
				</span>
			)}
		</>
	)
}
