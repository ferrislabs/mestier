import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCreateKnowledgeTable,
	useDeleteKnowledgeTable,
	useKnowledgeTables,
	useUpdateKnowledgeTable,
} from '#/hooks/use-knowledge-tables'
import { useHasPermission } from '#/hooks/use-permissions'
import { tableErrorMessage } from '#/pages/knowledge-tables/lib/errors'
import type { KnowledgeTable } from '#/pages/knowledge-tables/lib/types'
import {
	EMPTY_TABLE_DRAFT,
	TableDialog,
	type TableDraft,
} from '#/pages/knowledge-tables/ui/table-dialog'
import { TablesListPage } from '#/pages/knowledge-tables/ui/tables-list-page'

interface TablesListFeatureProps {
	onOpenTable: (tableId: string) => void
}

type Draft =
	| { mode: 'create'; values: TableDraft }
	| { mode: 'rename'; table: KnowledgeTable; values: TableDraft }

export function TablesListFeature({ onOpenTable }: TablesListFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const canManage = useHasPermission('MANAGE_KNOWLEDGE')
	const tables = useKnowledgeTables(organizationId)
	const createTable = useCreateKnowledgeTable(organizationId)
	const updateTable = useUpdateKnowledgeTable(organizationId)
	const deleteTable = useDeleteKnowledgeTable(organizationId)
	const [draft, setDraft] = useState<Draft | null>(null)
	const [toDelete, setToDelete] = useState<KnowledgeTable | null>(null)

	const saving = draft?.mode === 'rename' ? updateTable : createTable

	const submit = async () => {
		if (!draft) return
		const name = draft.values.name.trim()
		const description = draft.values.description.trim() || null
		try {
			if (draft.mode === 'create') {
				const created = await createTable.mutateAsync({ name, description })
				setDraft(null)
				onOpenTable(created.id)
			} else {
				await updateTable.mutateAsync({
					tableId: draft.table.id,
					name,
					description,
				})
				setDraft(null)
			}
		} catch {}
	}

	const confirmDelete = async () => {
		if (!toDelete) return
		try {
			await deleteTable.mutateAsync(toDelete.id)
			setToDelete(null)
		} catch {}
	}

	return (
		<>
			<TablesListPage
				tables={tables.data ?? []}
				canManage={canManage}
				isLoading={tables.isLoading}
				error={tableErrorMessage(tables.error)}
				onCreate={() => setDraft({ mode: 'create', values: EMPTY_TABLE_DRAFT })}
				onOpen={(table) => onOpenTable(table.id)}
				onEdit={(table) =>
					setDraft({
						mode: 'rename',
						table,
						values: { name: table.name, description: table.description ?? '' },
					})
				}
				onDelete={setToDelete}
			/>
			<TableDialog
				open={draft !== null}
				mode={draft?.mode ?? 'create'}
				draft={draft?.values ?? EMPTY_TABLE_DRAFT}
				error={tableErrorMessage(saving.error)}
				isSaving={saving.isPending}
				onDraftChange={(change) => {
					saving.reset()
					setDraft((current) =>
						current
							? { ...current, values: { ...current.values, ...change } }
							: current,
					)
				}}
				onSubmit={() => {
					void submit()
				}}
				onOpenChange={(open) => {
					if (!open) {
						setDraft(null)
						createTable.reset()
						updateTable.reset()
					}
				}}
			/>
			<ConfirmDialog
				open={toDelete !== null}
				onOpenChange={(open) => {
					if (!open) {
						setToDelete(null)
						deleteTable.reset()
					}
				}}
				title={`Supprimer « ${toDelete?.name ?? ''} » ?`}
				description="La table, ses colonnes, ses vues et toutes ses lignes sont effacées. Rien ne se récupère."
				confirmLabel="Supprimer la table"
				destructive
				disabled={deleteTable.isPending}
				onConfirm={() => {
					void confirmDelete()
				}}
			>
				{deleteTable.error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{tableErrorMessage(deleteTable.error)}
					</p>
				) : null}
			</ConfirmDialog>
		</>
	)
}
