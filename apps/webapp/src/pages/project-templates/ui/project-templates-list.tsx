import type { ColumnDef } from '@tanstack/react-table'
import { LayoutTemplate, Loader2, Plus } from 'lucide-react'
import { useMemo } from 'react'
import {
	ReferenceTable,
	RowActions,
	RowIdentity,
} from '#/components/reference-table'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
} from '#/components/ui/surface'
import type { ProjectTemplate } from '#/hooks/use-project-templates'

export interface ProjectTemplatesListProps {
	organizationName: string
	templates: ProjectTemplate[]
	includeArchived: boolean
	isLoading: boolean
	error: string | null
	onIncludeArchivedChange: (includeArchived: boolean) => void
	onCreate: () => void
	onEdit: (template: ProjectTemplate) => void
	onArchive: (template: ProjectTemplate) => void
	onRestore: (template: ProjectTemplate) => void
}

/**
 * The template list. Pure presentation: no hooks beyond `useMemo`, no
 * fetching, no mutation — mirrors `ProjectsList`.
 */
export function ProjectTemplatesList({
	organizationName,
	templates,
	includeArchived,
	isLoading,
	error,
	onIncludeArchivedChange,
	onCreate,
	onEdit,
	onArchive,
	onRestore,
}: ProjectTemplatesListProps) {
	const columns = useMemo<ColumnDef<ProjectTemplate>[]>(
		() => [
			{
				id: 'name',
				header: 'Modèle',
				cell: ({ row }) => (
					<div className="flex min-w-0 flex-col gap-1">
						<RowIdentity title={row.original.name} id={row.original.id} />
						{row.original.archived_at ? (
							<Badge variant="outline">Archivé</Badge>
						) : null}
					</div>
				),
			},
			{
				id: 'description',
				header: 'Description',
				cell: ({ row }) =>
					row.original.description ? (
						<span className="truncate">{row.original.description}</span>
					) : (
						<span className="text-muted-foreground italic">Aucune</span>
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
						<RowActions
							isEditing={false}
							isSaving={false}
							onEdit={() => onEdit(row.original)}
							onCancel={() => {}}
							onSave={() => {}}
							onDelete={() => onArchive(row.original)}
						/>
					),
			},
		],
		[onArchive, onEdit, onRestore],
	)

	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Modèles de projet"
				description="Un modèle est un ensemble de tâches types avec des décalages, pas un projet à copier. Instancier un modèle produit un vrai projet, avec de vraies tâches, sur une date choisie."
				actions={
					<Button onClick={onCreate}>
						<Plus />
						Nouveau modèle
					</Button>
				}
			/>

			<section className="grid grid-cols-2 gap-4 lg:grid-cols-3">
				<MetricCard
					label="Modèles"
					value={templates.length}
					icon={<LayoutTemplate className="size-4" />}
				/>
			</section>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="flex items-center gap-2">
				<input
					id="include-archived-templates"
					type="checkbox"
					className="size-4"
					checked={includeArchived}
					onChange={(event) => onIncludeArchivedChange(event.target.checked)}
				/>
				<Label htmlFor="include-archived-templates" className="text-sm">
					Afficher les modèles archivés
				</Label>
			</div>

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des modèles…
				</SectionCard>
			) : (
				<ReferenceTable
					title={`Modèles (${templates.length})`}
					description="Archiver un modèle le retire du picker sans effacer ce qu'il a déjà produit."
					emptyTitle="Aucun modèle"
					emptyDescription="Créez-en un pour arrêter de reconstruire la même liste de tâches à chaque chantier."
					data={templates}
					columns={columns}
				/>
			)}
		</PageShell>
	)
}
