import type { Schemas } from '#/api/api.client'
import {
	type EditorState,
	EMPTY_ESTABLISHMENT,
	type EstablishmentValues,
	toEstablishmentValues,
} from '#/pages/cursus/catalog/lib/catalog'
import { CatalogDialog } from '#/pages/cursus/catalog/ui/catalog-block'
import { EstablishmentForm } from '#/pages/cursus/catalog/ui/catalog-forms'
import {
	type CatalogColumn,
	type CatalogListState,
	CatalogTablePage,
} from '#/pages/cursus/catalog/ui/catalog-table-page'
import { ESTABLISHMENT_NOUN } from '#/pages/cursus/shared/lib/count-summary'
import { MASCULINE_ARCHIVE_OPTIONS } from '#/pages/cursus/shared/lib/list-filters'

type Establishment = Schemas.EstablishmentResponse

const COLUMNS: CatalogColumn<Establishment>[] = [
	{ header: 'UAI', cell: (item) => item.uai ?? '—' },
	{ header: 'SIRET', cell: (item) => item.siret ?? '—' },
	{
		header: 'Site principal',
		cell: (item) => (item.is_main_site ? 'Oui' : '—'),
	},
]

export interface EstablishmentsPageProps extends CatalogListState {
	items: Establishment[]
	editor: EditorState
	isSaving: boolean
	saveError: string | null
	onOpenEditor: (id: string | null) => void
	onCloseEditor: () => void
	onSave: (values: EstablishmentValues, id: string | null) => void
	onArchive: (id: string) => void
}

export function EstablishmentsPage({
	items,
	editor,
	isSaving,
	saveError,
	onOpenEditor,
	onCloseEditor,
	onSave,
	onArchive,
	...list
}: EstablishmentsPageProps) {
	const editing = items.find((item) => item.id === editor?.id)

	return (
		<CatalogTablePage
			{...list}
			title="Établissements"
			addLabel="Nouvel établissement"
			noun={ESTABLISHMENT_NOUN}
			searchPlaceholder="Rechercher un établissement, un UAI"
			archiveOptions={MASCULINE_ARCHIVE_OPTIONS}
			emptyTitle="Aucun établissement ne correspond."
			activeLabel="Actif"
			archivedLabel="Archivé"
			columns={COLUMNS}
			items={items}
			onAdd={() => onOpenEditor(null)}
			onEdit={onOpenEditor}
			onArchive={onArchive}
			dialog={
				list.canManage && editor ? (
					<CatalogDialog
						open
						title={
							editing ? "Modifier l'établissement" : 'Nouvel établissement'
						}
						description="Identité administrative de l'établissement."
						onOpenChange={(open) => {
							if (!open) onCloseEditor()
						}}
					>
						<EstablishmentForm
							initialValues={
								editing ? toEstablishmentValues(editing) : EMPTY_ESTABLISHMENT
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
