import { Link } from '@tanstack/react-router'
import { Folder } from 'lucide-react'
import { buildOrgPath } from '#/modules/org-path'
import { projectPath } from '#/pages/automation/lib/automation-paths'
import type { WorkflowFolder } from '#/pages/automation/lib/folders'

export function FolderRow({
	folder,
	workflowCount,
	organizationSlug,
}: {
	folder: WorkflowFolder
	workflowCount: number
	organizationSlug: string
}) {
	return (
		<li className="relative flex items-center gap-3 px-4 py-3 transition-colors hover:bg-muted/50">
			<Folder
				className="size-5 shrink-0 text-muted-foreground"
				strokeWidth={1.6}
				aria-hidden
			/>
			<div className="min-w-0">
				<Link
					to={buildOrgPath(organizationSlug, projectPath(folder.id))}
					className="block truncate font-medium text-base outline-none after:absolute after:inset-0 focus-visible:after:ring-2 focus-visible:after:ring-ring focus-visible:after:ring-inset"
				>
					{folder.name}
				</Link>
				<p className="mt-0.5 text-muted-foreground text-sm">
					{workflowCount === 0
						? 'Aucun workflow'
						: `${workflowCount} workflow${workflowCount > 1 ? 's' : ''}`}
				</p>
			</div>
		</li>
	)
}
