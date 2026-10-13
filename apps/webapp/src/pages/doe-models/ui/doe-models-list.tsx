import type { ColumnDef } from '@tanstack/react-table'
import { Loader2, Plus, Sparkles } from 'lucide-react'
import { useMemo } from 'react'
import {
	ReferenceTable,
	RowActions,
	RowIdentity,
} from '#/components/reference-table'
import { RequirePermission } from '#/components/require-permission'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import type { DoeModel } from '#/hooks/use-doe-models'

interface DoeModelsListProps {
	models: DoeModel[]
	includeArchived: boolean
	isLoading: boolean
	error: string | null
	onIncludeArchivedChange: (includeArchived: boolean) => void
	onCreate: () => void
	onOpenStarters: () => void
	onEdit: (model: DoeModel) => void
	onArchive: (model: DoeModel) => void
	onRestore: (model: DoeModel) => void
}

export function DoeModelsList({
	models,
	includeArchived,
	isLoading,
	error,
	onIncludeArchivedChange,
	onCreate,
	onOpenStarters,
	onEdit,
	onArchive,
	onRestore,
}: DoeModelsListProps) {
	const columns = useMemo<ColumnDef<DoeModel>[]>(
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
				id: 'sections',
				header: 'Sections',
				cell: ({ row }) => (
					<span>
						{row.original.sections.length} (
						{row.original.sections.filter((section) => section.enabled).length}{' '}
						cochées)
					</span>
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
							<RequirePermission permission="MANAGE_REFERENCE">
								<Button
									variant="outline"
									size="sm"
									onClick={() => onRestore(row.original)}
								>
									Restaurer
								</Button>
							</RequirePermission>
						</div>
					) : (
						<div className="flex items-center justify-end gap-2">
							<RowActions
								permission="MANAGE_REFERENCE"
								isEditing={false}
								isSaving={false}
								onEdit={() => onEdit(row.original)}
								onCancel={() => {}}
								onSave={() => {}}
								onDelete={() => onArchive(row.original)}
							/>
						</div>
					),
			},
		],
		[onArchive, onEdit, onRestore],
	)

	return (
		<div className="flex flex-col gap-4">
			<SectionHeader
				className="border-b-0 px-0 py-0"
				title="Modèles de DOE"
				description="Un modèle est une liste de thèmes pré-réglés. Pour l'utiliser, crée un DOE depuis la page d'un projet : ses thèmes y sont copiés et restent modifiables."
				actions={
					<RequirePermission permission="MANAGE_REFERENCE">
						<div className="flex flex-wrap gap-2">
							<Button variant="outline" onClick={onOpenStarters}>
								<Sparkles />
								Créer depuis un modèle de départ
							</Button>
							<Button onClick={onCreate}>
								<Plus />
								Nouveau modèle
							</Button>
						</div>
					</RequirePermission>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="flex items-center gap-2">
				<input
					id="include-archived-doe-models"
					type="checkbox"
					className="size-4"
					checked={includeArchived}
					onChange={(event) => onIncludeArchivedChange(event.target.checked)}
				/>
				<Label htmlFor="include-archived-doe-models" className="text-sm">
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
					title="Modèles"
					description="Archiver un modèle le retire de la liste proposée à la création d'un DOE, sans toucher aux DOE déjà créés."
					emptyTitle="Aucun modèle de DOE"
					emptyDescription="Créez-en un, ou partez du modèle Paysagiste."
					data={models}
					columns={columns}
				/>
			)}
		</div>
	)
}
