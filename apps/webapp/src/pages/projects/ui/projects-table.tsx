import { Link } from '@tanstack/react-router'
import { Progress } from '#/components/ui/progress'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { buildOrgPath } from '#/modules/org-path'
import type { ProjectRow } from '#/pages/projects/lib/project-list'
import { missionsRatio, periodLabel } from '#/pages/projects/lib/project-view'
import { PersonAvatar } from '#/pages/projects/ui/avatars'
import { ProjectStatusBadge } from '#/pages/projects/ui/badges'
import { Dash } from '#/pages/projects/ui/dash'
import { InternalChip } from '#/pages/projects/ui/project-chips'

export function ProjectsTable({
	organizationSlug,
	rows,
	highlightedProjectId,
}: {
	organizationSlug: string
	rows: readonly ProjectRow[]
	highlightedProjectId?: string
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Projet</TableHead>
						<TableHead className="max-md:hidden">Client</TableHead>
						<TableHead>Missions</TableHead>
						<TableHead className="max-lg:hidden">DOE</TableHead>
						<TableHead className="max-xl:hidden">Période</TableHead>
						<TableHead className="max-md:hidden">Responsable</TableHead>
						<TableHead className="pr-4">Statut</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{rows.map((row) => {
						const period = periodLabel(row.extras.startDate, row.extras.endDate)
						return (
							<TableRow
								key={row.project.id}
								data-highlighted={
									row.project.id === highlightedProjectId ? 'true' : undefined
								}
								className="relative cursor-pointer focus-within:bg-muted/50 data-[highlighted=true]:bg-primary/5"
							>
								<TableCell className="max-w-72 py-2.5 pl-4">
									<Link
										to={buildOrgPath(
											organizationSlug,
											'/planification/projects/$projectId',
										)}
										params={{ projectId: row.project.id }}
										className="block truncate font-medium outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
									>
										{row.project.name}
									</Link>
									<span className="block truncate text-[12px] text-muted-foreground">
										{row.extras.place || 'Lieu non renseigné'}
									</span>
								</TableCell>
								<TableCell className="max-md:hidden">
									{row.customerName ? (
										<span className="block max-w-48 truncate">
											{row.customerName}
										</span>
									) : (
										<InternalChip />
									)}
								</TableCell>
								<TableCell>
									{row.missions.total === 0 ? (
										<Dash label="Aucune mission" />
									) : (
										<div className="flex items-center gap-2">
											<Progress
												value={row.missions.progress}
												aria-label={`${row.missions.done} missions terminées sur ${row.missions.total}`}
												className="h-1.5 w-20"
											/>
											<span className="text-[13px] tabular-nums">
												{missionsRatio(row.missions)}
											</span>
										</div>
									)}
								</TableCell>
								<TableCell className="max-lg:hidden">
									{row.doeRevision ? (
										`Indice ${row.doeRevision}`
									) : (
										<span className="text-muted-foreground">Aucun</span>
									)}
								</TableCell>
								<TableCell className="text-muted-foreground max-xl:hidden">
									{period ?? <Dash label="Période non renseignée" />}
								</TableCell>
								<TableCell className="max-md:hidden">
									<span className="flex items-center gap-2">
										<PersonAvatar person={row.owner} />
										{row.owner ? (
											<span className="block max-w-36 truncate">
												{row.owner.name}
											</span>
										) : (
											<span className="text-muted-foreground">
												Sans responsable
											</span>
										)}
									</span>
								</TableCell>
								<TableCell className="pr-4">
									<ProjectStatusBadge status={row.status} />
								</TableCell>
							</TableRow>
						)
					})}
				</TableBody>
			</Table>
		</div>
	)
}
