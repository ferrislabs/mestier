import type { Schemas } from '#/api/api.client'
import {
	type EditorState,
	EMPTY_TRAINING,
	type TrainingValues,
	toTrainingValues,
} from '#/pages/cursus/catalog/lib/catalog'
import { CatalogDialog } from '#/pages/cursus/catalog/ui/catalog-block'
import { TrainingForm } from '#/pages/cursus/catalog/ui/catalog-forms'
import {
	type CatalogColumn,
	type CatalogListState,
	CatalogTablePage,
} from '#/pages/cursus/catalog/ui/catalog-table-page'
import { TRAINING_NOUN } from '#/pages/cursus/shared/lib/count-summary'
import { FEMININE_ARCHIVE_OPTIONS } from '#/pages/cursus/shared/lib/list-filters'

type Training = Schemas.TrainingResponse

const COLUMNS: CatalogColumn<Training>[] = [
	{ header: 'Code RNCP', cell: (item) => item.rncp_code ?? '—' },
	{ header: 'Niveau', cell: (item) => item.level ?? '—' },
	{ header: 'Diplôme', cell: (item) => item.degree_type ?? '—' },
]

export interface TrainingsPageProps extends CatalogListState {
	items: Training[]
	editor: EditorState
	isSaving: boolean
	saveError: string | null
	onOpenEditor: (id: string | null) => void
	onCloseEditor: () => void
	onSave: (values: TrainingValues, id: string | null) => void
	onArchive: (id: string) => void
}

export function TrainingsPage({
	items,
	editor,
	isSaving,
	saveError,
	onOpenEditor,
	onCloseEditor,
	onSave,
	onArchive,
	...list
}: TrainingsPageProps) {
	const editing = items.find((item) => item.id === editor?.id)

	return (
		<CatalogTablePage
			{...list}
			title="Formations"
			addLabel="Nouvelle formation"
			noun={TRAINING_NOUN}
			searchPlaceholder="Rechercher une formation, un code RNCP"
			archiveOptions={FEMININE_ARCHIVE_OPTIONS}
			emptyTitle="Aucune formation ne correspond."
			activeLabel="Active"
			archivedLabel="Archivée"
			columns={COLUMNS}
			items={items}
			onAdd={() => onOpenEditor(null)}
			onEdit={onOpenEditor}
			onArchive={onArchive}
			dialog={
				list.canManage && editor ? (
					<CatalogDialog
						open
						title={editing ? 'Modifier la formation' : 'Nouvelle formation'}
						description="Intitulé et référentiel RNCP de la formation."
						onOpenChange={(open) => {
							if (!open) onCloseEditor()
						}}
					>
						<TrainingForm
							initialValues={
								editing ? toTrainingValues(editing) : EMPTY_TRAINING
							}
							isPending={isSaving}
							error={saveError}
							onSubmit={(values) => onSave(values, editor.id)}
						/>
					</CatalogDialog>
				) : null
			}
		/>
	)
}
