import { Link } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

const LINK =
	'rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring'

export function DafBreadcrumb({
	organizationSlug,
	projectId,
	projectName,
	current,
}: {
	organizationSlug: string
	projectId: string
	projectName: string
	current?: string
}) {
	return (
		<nav aria-label="Chemin des DAF" className="mb-1">
			<ol className="flex flex-wrap items-center gap-1.5 font-semibold text-[11px] text-muted-foreground uppercase tracking-wide">
				<li>
					<Link to={buildOrgPath(organizationSlug, '/projet')} className={LINK}>
						Projets
					</Link>
				</li>
				<li aria-hidden>/</li>
				<li className="min-w-0 truncate">
					<Link
						to={buildOrgPath(organizationSlug, '/projet/$projectId')}
						params={{ projectId }}
						className={LINK}
					>
						{projectName}
					</Link>
				</li>
				<li aria-hidden>/</li>
				{current ? (
					<>
						<li>
							<Link
								to={buildOrgPath(organizationSlug, '/projet/$projectId/daf')}
								params={{ projectId }}
								className={LINK}
							>
								DAF
							</Link>
						</li>
						<li aria-hidden>/</li>
						<li aria-current="page" className="min-w-0 truncate">
							{current}
						</li>
					</>
				) : (
					<li aria-current="page">DAF</li>
				)}
			</ol>
		</nav>
	)
}
