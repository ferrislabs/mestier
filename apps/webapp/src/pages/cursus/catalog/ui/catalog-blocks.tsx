import type { Schemas } from '#/api/api.client'
import {
	type ClosureReasonValues,
	type EditorState,
	EMPTY_CLOSURE_REASON,
} from '#/pages/cursus/catalog/lib/catalog'
import {
	CatalogBlock,
	CatalogDialog,
	CatalogRow,
} from '#/pages/cursus/catalog/ui/catalog-block'
import { ClosureReasonForm } from '#/pages/cursus/catalog/ui/catalog-forms'

interface BlockProps<Item, Values> {
	items: Item[]
	isLoading: boolean
	error: string | null
	canManage: boolean
	editor: EditorState
	isSaving: boolean
	saveError: string | null
	onOpenEditor: (id: string | null) => void
	onCloseEditor: () => void
	onSave: (values: Values, id: string | null) => void
	onArchive: (id: string) => void
}

export function ClosureReasonsBlock({
	items,
	isLoading,
	error,
	canManage,
	editor,
	isSaving,
	saveError,
	onOpenEditor,
	onCloseEditor,
	onSave,
	onArchive,
}: BlockProps<Schemas.ClosureReasonResponse, ClosureReasonValues>) {
	const editing = items.find((item) => item.id === editor?.id)

	return (
		<CatalogBlock
			title="Motifs de clôture"
			addLabel="Nouveau motif"
			canManage={canManage}
			isLoading={isLoading}
			error={error}
			isEmpty={items.length === 0}
			emptyLabel="Aucun motif de clôture."
			onAdd={() => onOpenEditor(null)}
		>
			{items.map((item) => (
				<CatalogRow
					key={item.id}
					title={item.label}
					archived={Boolean(item.archived_at)}
					canManage={canManage}
					onEdit={() => onOpenEditor(item.id)}
					onArchive={() => onArchive(item.id)}
				/>
			))}
			{canManage && editor ? (
				<CatalogDialog
					open
					title={editing ? 'Modifier le motif' : 'Nouveau motif de clôture'}
					description="Motif proposé lors de la clôture d'un trinôme ou d'un contrat."
					onOpenChange={(open) => {
						if (!open) onCloseEditor()
					}}
				>
					<ClosureReasonForm
						initialValues={
							editing ? { label: editing.label } : EMPTY_CLOSURE_REASON
						}
						isPending={isSaving}
						error={saveError}
						onSubmit={(values) => onSave(values, editor.id)}
					/>
				</CatalogDialog>
			) : null}
		</CatalogBlock>
	)
}
