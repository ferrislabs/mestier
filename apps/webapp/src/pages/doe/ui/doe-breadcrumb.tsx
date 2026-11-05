import { Link } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

const LINK =
	'rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring'

export function DoeBreadcrumb({
	organizationSlug,
	projectId,
	projectName,
}: {
	organizationSlug: string
	projectId: string
	projectName: string
}) {
	return (
		<nav aria-label="Chemin du DOE" className="mb-1">
			<ol className="flex flex-wrap items-center gap-1.5 font-semibold text-[11px] text-muted-foreground uppercase tracking-wide">
				<li>
					<Link
						to={buildOrgPath(organizationSlug, '/planification/projects')}
						className={LINK}
					>
						Projets
					</Link>
				</li>
				<li aria-hidden>/</li>
				<li className="min-w-0 truncate">
					<Link
						to={buildOrgPath(
							organizationSlug,
							'/planification/projects/$projectId',
						)}
						params={{ projectId }}
						className={LINK}
					>
						{projectName}
					</Link>
				</li>
				<li aria-hidden>/</li>
				<li aria-current="page">DOE</li>
			</ol>
		</nav>
	)
}
