import { Link } from '@tanstack/react-router'
import { LayoutTemplate, Loader2, Plus } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import {
	DEFAULT_FILTERS,
	hasActiveFilters,
	type ListEmpty,
	type ProjectFilters,
	type ProjectRow,
	type ProjectTiles,
} from '#/pages/projects/lib/project-list'
import { Page, PageHeader } from '#/pages/projects/ui/page'
import { ProjectsTable } from '#/pages/projects/ui/projects-table'
import { ProjectsToolbar } from '#/pages/projects/ui/projects-toolbar'
import { StatTile } from '#/pages/projects/ui/stat-tile'

const EMPTY_COPY: Record<ListEmpty, { title: string; hint: string }> = {
	none: {
		title: 'Aucun projet pour l’instant',
		hint: 'Crée ton premier projet pour suivre ses missions et préparer son DOE.',
	},
	'no-active': {
		title: 'Aucun projet actif',
		hint: 'Tous tes projets sont archivés. Affiche les archivés pour en réactiver un, ou crée un nouveau projet.',
	},
	'no-archived': {
		title: 'Aucun projet archivé',
		hint: 'Les projets que tu archives apparaissent ici et peuvent être réactivés.',
	},
	'no-match': {
		title: 'Aucun projet ne correspond',
		hint: 'Vérifie l’orthographe, le statut ou le client sélectionné.',
	},
}

export interface ProjectsListProps {
	organizationSlug: string
	summary: string
	tiles: ProjectTiles
	filters: ProjectFilters
	customers: readonly { id: string; label: string }[]
	listed: readonly ProjectRow[]
	empty: ListEmpty
	newProjectCustomerId: string | undefined
	highlightedProjectId?: string
	isLoading: boolean
	error: string | null
	onFiltersChange: (filters: ProjectFilters) => void
	onStartFromTemplate: () => void
}

export function ProjectsList({
	organizationSlug,
	summary,
	tiles,
	filters,
	customers,
	listed,
	empty,
	newProjectCustomerId,
	highlightedProjectId,
	isLoading,
	error,
	onFiltersChange,
	onStartFromTemplate,
}: ProjectsListProps) {
	const copy = EMPTY_COPY[empty]
	const newProjectPath = buildOrgPath(
		organizationSlug,
		'/planification/projects/new',
	)
	return (
		<Page>
			<PageHeader
				title="Projets"
				description={summary}
				actions={
					<RequirePermission permission="MANAGE_PLANNING">
						<Button variant="outline" size="sm" onClick={onStartFromTemplate}>
							<LayoutTemplate aria-hidden /> Depuis un modèle
						</Button>
						<Button size="sm" asChild>
							<Link
								to={newProjectPath}
								search={
									newProjectCustomerId ? { client: newProjectCustomerId } : {}
								}
							>
								<Plus aria-hidden /> Nouveau projet
							</Link>
						</Button>
					</RequirePermission>
				}
			/>
			<div className="mb-5 grid gap-3 sm:grid-cols-3">
				<StatTile
					label="Projets actifs"
					value={tiles.active}
					hint={
						tiles.internal > 0
							? `dont ${tiles.internal} en interne`
							: 'Aucun projet interne'
					}
				/>
				<StatTile
					label="Missions en cours"
					value={tiles.missionsInProgress}
					hint="Sur les projets actifs"
				/>
				<StatTile
					label="DOE à créer ou à finaliser"
					value={tiles.doeToCreate}
					hint="Projets actifs sans DOE"
				/>
			</div>
			{error ? (
				<div
					role="alert"
					className="mb-3 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
				>
					{error}
				</div>
			) : null}
			<ProjectsToolbar
				filters={filters}
				customers={customers}
				onChange={onFiltersChange}
			/>
			{isLoading ? (
				<div className="flex min-h-48 items-center justify-center gap-3 rounded-lg border bg-card p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" aria-hidden />
					Chargement des projets…
				</div>
			) : listed.length === 0 ? (
				<EmptyState title={copy.title} hint={copy.hint}>
					<div className="flex flex-wrap justify-center gap-2">
						{empty === 'no-match' && hasActiveFilters(filters) && (
							<Button
								variant="outline"
								size="sm"
								onClick={() => onFiltersChange(DEFAULT_FILTERS)}
							>
								Réinitialiser les filtres
							</Button>
						)}
						{empty === 'no-active' && (
							<Button
								variant="outline"
								size="sm"
								onClick={() =>
									onFiltersChange({ ...filters, status: 'archives' })
								}
							>
								Voir les archivés
							</Button>
						)}
						{(empty === 'none' || empty === 'no-active') && (
							<RequirePermission permission="MANAGE_PLANNING">
								<Button size="sm" asChild>
									<Link to={newProjectPath}>
										<Plus aria-hidden /> Nouveau projet
									</Link>
								</Button>
							</RequirePermission>
						)}
					</div>
				</EmptyState>
			) : (
				<ProjectsTable
					organizationSlug={organizationSlug}
					rows={listed}
					highlightedProjectId={highlightedProjectId}
				/>
			)}
		</Page>
	)
}
