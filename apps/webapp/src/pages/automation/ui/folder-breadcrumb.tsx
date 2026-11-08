import { Link } from '@tanstack/react-router'
import { ChevronRight } from 'lucide-react'
import { Fragment, type ReactNode } from 'react'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	projectPath,
	workflowsPath,
} from '#/pages/automation/lib/automation-paths'
import type { WorkflowFolder } from '#/pages/automation/lib/folders'
import { useDropTarget } from '#/pages/automation/ui/use-drop-target'

const LINK_CLASS =
	'rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring'

function Crumb({
	accepts,
	onDrop,
	children,
}: {
	accepts: boolean
	onDrop: () => void
	children: ReactNode
}) {
	const drop = useDropTarget({ accepts, onDrop })
	return (
		<li
			{...drop.handlers}
			className={cn(
				'rounded-sm px-1',
				accepts && 'outline-dashed outline-1 outline-border',
				drop.over && 'bg-accent/60 outline-ring',
			)}
		>
			{children}
		</li>
	)
}

export function FolderBreadcrumb({
	path,
	organizationSlug,
	canDrop,
	onDropItem,
}: {
	path: readonly WorkflowFolder[]
	organizationSlug: string
	canDrop: (targetId: string | null) => boolean
	onDropItem: (targetId: string | null) => void
}) {
	return (
		<nav aria-label="Emplacement du projet" className="mb-2">
			<ol className="flex flex-wrap items-center gap-1 text-muted-foreground text-sm">
				<Crumb accepts={canDrop(null)} onDrop={() => onDropItem(null)}>
					<Link
						to={buildOrgPath(organizationSlug, workflowsPath())}
						draggable={false}
						className={LINK_CLASS}
					>
						Workflows
					</Link>
				</Crumb>
				{path.map((folder, index) => (
					<Fragment key={folder.id}>
						<ChevronRight className="size-3.5" aria-hidden />
						<Crumb
							accepts={canDrop(folder.id)}
							onDrop={() => onDropItem(folder.id)}
						>
							{index === path.length - 1 ? (
								<span aria-current="page" className="text-foreground">
									{folder.name}
								</span>
							) : (
								<Link
									to={buildOrgPath(organizationSlug, projectPath(folder.id))}
									draggable={false}
									className={LINK_CLASS}
								>
									{folder.name}
								</Link>
							)}
						</Crumb>
					</Fragment>
				))}
			</ol>
		</nav>
	)
}
