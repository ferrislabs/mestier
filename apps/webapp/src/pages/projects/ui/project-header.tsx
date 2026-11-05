import { Link } from '@tanstack/react-router'
import { ArrowLeft } from 'lucide-react'
import type { ReactNode } from 'react'
import { buildOrgPath } from '#/modules/org-path'
import type { ProjectStatus } from '#/pages/projects/lib/model'
import { ProjectStatusBadge } from '#/pages/projects/ui/badges'
import { CustomerChip } from '#/pages/projects/ui/project-chips'

export function ProjectHeader({
	organizationSlug,
	name,
	status,
	customerName,
	actions,
}: {
	organizationSlug: string
	name: string
	status: ProjectStatus
	customerName: string | null
	actions?: ReactNode
}) {
	return (
		<header className="mb-6 flex flex-wrap items-end justify-between gap-4">
			<div className="min-w-0">
				<nav aria-label="Fil d’Ariane du projet">
					<ol className="flex items-center gap-1.5 font-semibold text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
						<li>
							<Link
								to={buildOrgPath(organizationSlug, '/projet')}
								className="rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
							>
								Projets
							</Link>
						</li>
						<li aria-hidden>/</li>
						<li aria-current="page" className="min-w-0 truncate">
							{name}
						</li>
					</ol>
				</nav>
				<h1 className="mt-1 break-words font-semibold text-2xl tracking-tight">
					{name}
				</h1>
				<div className="mt-2 flex flex-wrap items-center gap-2">
					<CustomerChip
						customer={customerName ? { name: customerName } : null}
					/>
					<ProjectStatusBadge status={status} />
				</div>
			</div>
			{actions && (
				<div className="flex flex-wrap items-center gap-2">{actions}</div>
			)}
		</header>
	)
}

export function ProjectSubHeader({
	organizationSlug,
	projectId,
	name,
	status,
	title,
	actions,
}: {
	organizationSlug: string
	projectId: string
	name: string
	status: ProjectStatus
	title: string
	actions?: ReactNode
}) {
	return (
		<header className="mb-6 flex flex-wrap items-end justify-between gap-4">
			<div className="min-w-0">
				<Link
					to={buildOrgPath(organizationSlug, '/projet/$projectId')}
					params={{ projectId }}
					className="inline-flex items-center gap-1 rounded-sm font-medium text-[13px] text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
				>
					<ArrowLeft className="size-3.5" aria-hidden />
					Aperçu du projet
				</Link>
				<h1 className="mt-1 font-semibold text-2xl tracking-tight">{title}</h1>
				<p className="mt-1 flex flex-wrap items-center gap-2 text-sm">
					<span className="min-w-0 truncate font-medium">{name}</span>
					<ProjectStatusBadge status={status} />
				</p>
			</div>
			{actions && <div className="flex items-center gap-2">{actions}</div>}
		</header>
	)
}
