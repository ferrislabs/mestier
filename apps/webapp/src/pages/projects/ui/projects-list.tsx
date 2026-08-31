import type { ColumnDef } from '@tanstack/react-table'
import {
	Building2,
	FolderKanban,
	LayoutTemplate,
	Loader2,
	Plus,
	Search,
} from 'lucide-react'
import { useMemo } from 'react'
import {
	ReferenceTable,
	RowActions,
	RowIdentity,
} from '#/components/reference-table'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
} from '#/components/ui/surface'
import type { Project } from '#/hooks/use-projects'
import { cn } from '#/lib/utils'

export interface ProjectsListProps {
	projects: Project[]
	/** Resolves a customer id into a name, or `null` while unknown. */
	customerName: (customerId: string) => string | null
	search: string
	includeArchived: boolean
	/** The row the profitability screen linked at, highlighted once. */
	highlightedProjectId?: string
	isLoading: boolean
	error: string | null
	editingId: string | null
	isSaving: boolean
	/** True while a project's tasks are being loaded to seed "save as template". */
	isBuildingTemplate: boolean
	onSearchChange: (search: string) => void
	onIncludeArchivedChange: (includeArchived: boolean) => void
	onCreate: () => void
	onStartFromTemplate: () => void
	onEdit: (project: Project) => void
	onCancelEdit: () => void
	onSaveEdit: () => void
	onArchive: (project: Project) => void
	onRestore: (project: Project) => void
	onSaveAsTemplate: (project: Project) => void
}

/**
 * The project list. Pure presentation: no hooks beyond `useMemo` over the props
 * it is handed, no fetching, no mutation.
 */
export function ProjectsList({
	projects,
	customerName,
	search,
	includeArchived,
	highlightedProjectId,
	isLoading,
	error,
	editingId,
	isSaving,
	isBuildingTemplate,
	onSearchChange,
	onIncludeArchivedChange,
	onCreate,
	onStartFromTemplate,
	onEdit,
	onCancelEdit,
	onSaveEdit,
	onArchive,
	onRestore,
	onSaveAsTemplate,
}: ProjectsListProps) {
	const internalCount = projects.filter((project) => project.is_internal).length

	const columns = useMemo<ColumnDef<Project>[]>(
		() => [
			{
				id: 'name',
				header: 'Projet',
				cell: ({ row }) => (
					<div
						className={cn(
							'flex min-w-0 flex-col gap-1',
							row.original.id === highlightedProjectId &&
								'-mx-2 rounded-md bg-primary/5 px-2 py-1',
						)}
					>
						<RowIdentity title={row.original.name} id={row.original.id} />
						<div className="flex flex-wrap gap-1">
							{row.original.is_internal ? (
								<Badge variant="secondary" className="gap-1">
									<Building2 className="size-3" />
									Interne
								</Badge>
							) : null}
							{row.original.archived_at ? (
								<Badge variant="outline">Archivé</Badge>
							) : null}
						</div>
					</div>
				),
			},
			{
				id: 'customer',
				header: 'Client',
				cell: ({ row }) =>
					row.original.customer_id ? (
						<span className="truncate">
							{customerName(row.original.customer_id) ?? 'Client'}
						</span>
					) : (
						<span className="text-muted-foreground italic">
							Aucun, projet interne
						</span>
					),
			},
			{
				id: 'quote',
				header: 'Devis',
				cell: ({ row }) =>
					row.original.quote_id ? (
						<span className="font-mono text-xs">{row.original.quote_id}</span>
					) : (
						<span className="text-muted-foreground italic">
							Aucun, pas de marge
						</span>
					),
			},
			{
				id: 'actions',
				header: () => <span className="sr-only">Actions</span>,
				cell: ({ row }) =>
					row.original.archived_at ? (
						<div className="flex justify-end">
							<Button
								variant="outline"
								size="sm"
								onClick={() => onRestore(row.original)}
							>
								Restaurer
							</Button>
						</div>
					) : (
						<div className="flex items-center justify-end gap-1">
							<Button
								variant="ghost"
								size="icon"
								disabled={isBuildingTemplate}
								onClick={() => onSaveAsTemplate(row.original)}
								aria-label="Enregistrer comme modèle"
								title="Enregistrer comme modèle"
							>
								<LayoutTemplate className="size-4" />
							</Button>
							<RowActions
								isEditing={editingId === row.original.id}
								isSaving={isSaving}
								onEdit={() => onEdit(row.original)}
								onCancel={onCancelEdit}
								onSave={onSaveEdit}
								onDelete={() => onArchive(row.original)}
							/>
						</div>
					),
			},
		],
		[
			customerName,
			editingId,
			highlightedProjectId,
			isBuildingTemplate,
			isSaving,
			onArchive,
			onCancelEdit,
			onEdit,
			onRestore,
			onSaveAsTemplate,
			onSaveEdit,
		],
	)

	return (
		<PageShell>
			<PageHeader
				title="Projets"
				description="Le sujet auquel un coût est rattaché. Un projet sans client est un projet interne : une réunion récurrente coûte, et doit se voir."
				actions={
					<div className="flex gap-2">
						<Button variant="outline" onClick={onStartFromTemplate}>
							<LayoutTemplate />
							Depuis un modèle
						</Button>
						<Button onClick={onCreate}>
							<Plus />
							Nouveau projet
						</Button>
					</div>
				}
			/>

			<section className="grid grid-cols-2 gap-4 lg:grid-cols-3">
				<MetricCard
					label="Projets"
					value={projects.length}
					icon={<FolderKanban className="size-4" />}
				/>
				<MetricCard
					label="Internes"
					value={internalCount}
					hint="Sans client, coûtés quand même"
					icon={<Building2 className="size-4" />}
				/>
			</section>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
				<div className="relative w-full lg:w-80">
					<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
					<Input
						type="search"
						value={search}
						onChange={(event) => onSearchChange(event.target.value)}
						placeholder="Rechercher un projet…"
						className="pl-9"
					/>
				</div>
				<div className="flex items-center gap-2">
					<input
						id="include-archived"
						type="checkbox"
						className="size-4"
						checked={includeArchived}
						onChange={(event) => onIncludeArchivedChange(event.target.checked)}
					/>
					<Label htmlFor="include-archived" className="text-sm">
						Afficher les projets archivés
					</Label>
				</div>
			</div>

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des projets…
				</SectionCard>
			) : (
				<ReferenceTable
					title={`Projets (${projects.length})`}
					description="Archiver un projet le retire des listes sans effacer son historique de coût."
					emptyTitle="Aucun projet"
					emptyDescription="Créez-en un pour rattacher des tâches et voir ce qu'elles coûtent."
					data={projects}
					columns={columns}
				/>
			)}
		</PageShell>
	)
}
