import type { Schemas } from '#/api/api.client'
import {
	type ClosureReasonValues,
	type EditorState,
	EMPTY_CLOSURE_REASON,
	EMPTY_ESTABLISHMENT,
	EMPTY_TRAINING,
	type EstablishmentValues,
	type TrainingValues,
	toEstablishmentValues,
	toTrainingValues,
} from '#/pages/cursus/catalog/lib/catalog'
import {
	CatalogBlock,
	CatalogDialog,
	type CatalogPageLayout,
	CatalogRow,
} from '#/pages/cursus/catalog/ui/catalog-block'
import {
	ClosureReasonForm,
	EstablishmentForm,
	TrainingForm,
} from '#/pages/cursus/catalog/ui/catalog-forms'

interface BlockProps<Item, Values> {
	page?: CatalogPageLayout
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

export function EstablishmentsBlock({
	page,
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
}: BlockProps<Schemas.EstablishmentResponse, EstablishmentValues>) {
	const editing = items.find((item) => item.id === editor?.id)

	return (
		<CatalogBlock
			page={page}
			title="Établissements"
			addLabel="Nouvel établissement"
			canManage={canManage}
			isLoading={isLoading}
			error={error}
			isEmpty={items.length === 0}
			emptyLabel="Aucun établissement."
			onAdd={() => onOpenEditor(null)}
		>
			{items.map((item) => (
				<CatalogRow
					key={item.id}
					title={item.name}
					subtitle={[item.uai, item.siret].filter(Boolean).join(' · ')}
					badge={item.is_main_site ? 'Site principal' : undefined}
					archived={Boolean(item.archived_at)}
					canManage={canManage}
					onEdit={() => onOpenEditor(item.id)}
					onArchive={() => onArchive(item.id)}
				/>
			))}
			{canManage && editor ? (
				<CatalogDialog
					open
					title={editing ? "Modifier l'établissement" : 'Nouvel établissement'}
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
			) : null}
		</CatalogBlock>
	)
}

export function TrainingsBlock({
	page,
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
}: BlockProps<Schemas.TrainingResponse, TrainingValues>) {
	const editing = items.find((item) => item.id === editor?.id)

	return (
		<CatalogBlock
			page={page}
			title="Formations"
			addLabel="Nouvelle formation"
			canManage={canManage}
			isLoading={isLoading}
			error={error}
			isEmpty={items.length === 0}
			emptyLabel="Aucune formation."
			onAdd={() => onOpenEditor(null)}
		>
			{items.map((item) => (
				<CatalogRow
					key={item.id}
					title={item.name}
					subtitle={[item.rncp_code, item.level, item.degree_type]
						.filter(Boolean)
						.join(' · ')}
					archived={Boolean(item.archived_at)}
					canManage={canManage}
					onEdit={() => onOpenEditor(item.id)}
					onArchive={() => onArchive(item.id)}
				/>
			))}
			{canManage && editor ? (
				<CatalogDialog
					open
					title={editing ? 'Modifier la formation' : 'Nouvelle formation'}
					description="Intitulé et référentiel RNCP de la formation."
					onOpenChange={(open) => {
						if (!open) onCloseEditor()
					}}
				>
					<TrainingForm
						initialValues={editing ? toTrainingValues(editing) : EMPTY_TRAINING}
						isPending={isSaving}
						error={saveError}
						onSubmit={(values) => onSave(values, editor.id)}
					/>
				</CatalogDialog>
			) : null}
		</CatalogBlock>
	)
}

export function ClosureReasonsBlock({
	page,
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
			page={page}
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
