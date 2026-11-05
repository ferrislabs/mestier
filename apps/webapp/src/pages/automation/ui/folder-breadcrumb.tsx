import { Link } from '@tanstack/react-router'
import { ChevronRight } from 'lucide-react'
import { Fragment } from 'react'
import { buildOrgPath } from '#/modules/org-path'
import {
	projectPath,
	workflowsPath,
} from '#/pages/automation/lib/automation-paths'
import type { WorkflowFolder } from '#/pages/automation/lib/folders'

export function FolderBreadcrumb({
	path,
	organizationSlug,
}: {
	path: readonly WorkflowFolder[]
	organizationSlug: string
}) {
	return (
		<nav aria-label="Emplacement du projet" className="mb-2">
			<ol className="flex flex-wrap items-center gap-1 text-muted-foreground text-sm">
				<li>
					<Link
						to={buildOrgPath(organizationSlug, workflowsPath())}
						className="rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
					>
						Workflows
					</Link>
				</li>
				{path.map((folder, index) => (
					<Fragment key={folder.id}>
						<ChevronRight className="size-3.5" aria-hidden />
						<li>
							{index === path.length - 1 ? (
								<span aria-current="page" className="text-foreground">
									{folder.name}
								</span>
							) : (
								<Link
									to={buildOrgPath(organizationSlug, projectPath(folder.id))}
									className="rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
								>
									{folder.name}
								</Link>
							)}
						</li>
					</Fragment>
				))}
			</ol>
		</nav>
	)
}
