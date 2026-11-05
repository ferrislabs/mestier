import { Link } from '@tanstack/react-router'
import { Folder, FolderOpen } from 'lucide-react'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { WorkflowFolder } from '#/pages/automation/lib/folders'

export interface ProjectsSidebarEntry {
	folder: WorkflowFolder
	workflowCount: number
	active: boolean
}

export function ProjectsSidebar({
	organizationSlug,
	entries,
}: {
	organizationSlug: string
	entries: readonly ProjectsSidebarEntry[]
}) {
	return (
		<nav aria-labelledby="automation-projects-heading" className="mt-4">
			<h2
				id="automation-projects-heading"
				className="px-2.5 pb-1 font-medium text-nav-muted text-xs"
			>
				Projets
			</h2>
			<ul className="flex flex-col gap-px">
				{entries.map(({ folder, workflowCount, active }) => {
					const Icon = active ? FolderOpen : Folder
					return (
						<li key={folder.id}>
							<Link
								to={buildOrgPath(
									organizationSlug,
									`/automatisation/projets/${folder.id}`,
								)}
								aria-current={active ? 'page' : undefined}
								className={cn(
									'flex h-8 items-center gap-2.5 rounded-md px-2.5 text-[13px] text-nav-muted outline-none transition-colors hover:bg-nav-active hover:text-nav-foreground focus-visible:ring-2 focus-visible:ring-ring',
									active &&
										'bg-nav-active font-medium text-nav-active-foreground',
								)}
							>
								<Icon
									className="size-4 shrink-0"
									strokeWidth={1.8}
									aria-hidden
								/>
								<span className="truncate">{folder.name}</span>
								<span className="tabular ml-auto text-[11px] text-nav-muted">
									{workflowCount}
								</span>
							</Link>
						</li>
					)
				})}
			</ul>
		</nav>
	)
}
