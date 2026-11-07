import { Plus } from 'lucide-react'
import type { ReactNode } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import {
	type CatalogFilters,
	type CatalogForm,
	type CatalogRow,
	type CatalogUsage,
	DEFAULT_FILTERS,
	IMMUTABLE_NOTE,
	isFiltered,
} from '#/pages/catalog/catalog-model'
import type { CatalogExtras } from '#/pages/catalog/feature/mocks/catalog-extras'
import {
	type CatalogEditor,
	CatalogFormDialog,
} from '#/pages/catalog/ui/catalog-form-dialog'
import { CatalogTable } from '#/pages/catalog/ui/catalog-table'
import { CatalogToolbar } from '#/pages/catalog/ui/catalog-toolbar'

interface CatalogUIProps {
	summary: string
	totalRows: number
	rows: readonly CatalogRow[]
	filters: CatalogFilters
	categories: readonly string[]
	onFiltersChange: (filters: CatalogFilters) => void
	extrasOf: (itemId: string) => CatalogExtras
	usageOf: (itemId: string) => CatalogUsage
	canManage: boolean
	isLoading: boolean
	error: string | null
	editor: CatalogEditor
	editing: boolean
	formInitial: CatalogForm
	customUnits: string[]
	isSaving: boolean
	saveError: string | null
	sameReference: (reference: string) => boolean
	onOpenEditor: (editor: CatalogEditor) => void
	onCloseEditor: () => void
	onCreateCustomUnit: (code: string) => void
	isCreatingCustomUnit: boolean
	onSubmit: (form: CatalogForm) => Promise<void>
	productExtras: (
		form: CatalogForm,
		change: (patch: Partial<CatalogForm>) => void,
	) => ReactNode
	onToggleArchive: (row: CatalogRow) => void
	deleting: CatalogRow | null
	onDeleteRequest: (row: CatalogRow | null) => void
	onDeleteConfirm: (row: CatalogRow) => void
}

export function CatalogUI({
	summary,
	totalRows,
	rows,
	filters,
	categories,
	onFiltersChange,
	extrasOf,
	usageOf,
	canManage,
	isLoading,
	error,
	editor,
	editing,
	formInitial,
	customUnits,
	isSaving,
	saveError,
	sameReference,
	onOpenEditor,
	onCloseEditor,
	onCreateCustomUnit,
	isCreatingCustomUnit,
	onSubmit,
	productExtras,
	onToggleArchive,
	deleting,
	onDeleteRequest,
	onDeleteConfirm,
}: CatalogUIProps) {
	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6 flex-row flex-wrap items-end justify-between gap-4"
				title="Catalogue"
				description={summary}
				actions={
					<RequirePermission permission="MANAGE_REFERENCE">
						<Button size="sm" onClick={() => onOpenEditor({ mode: 'create' })}>
							<Plus aria-hidden /> Nouvel article
						</Button>
					</RequirePermission>
				}
			/>
			<p className="-mt-3 mb-5 text-[13px] text-muted-foreground">
				{IMMUTABLE_NOTE}
			</p>
			<CatalogToolbar
				filters={filters}
				categories={categories}
				onChange={onFiltersChange}
			/>
			{error ? (
				<div className="mb-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-[13px] font-medium text-state-alert">
					{error}
				</div>
			) : null}
			{isLoading ? (
				<div className="rounded-lg border bg-card px-6 py-12 text-center text-[13px] text-muted-foreground">
					Chargement du catalogue…
				</div>
			) : rows.length === 0 ? (
				<EmptyState
					title={
						totalRows === 0
							? 'Le catalogue est vide.'
							: 'Aucun article ne correspond.'
					}
					hint={
						totalRows === 0
							? 'Crée ton premier service ou produit pour le retrouver dans tes devis et factures.'
							: 'Vérifie l’orthographe, le type ou la catégorie sélectionnée.'
					}
				>
					{isFiltered(filters) ? (
						<Button
							variant="outline"
							size="sm"
							onClick={() => onFiltersChange(DEFAULT_FILTERS)}
						>
							Réinitialiser les filtres
						</Button>
					) : (
						<RequirePermission permission="MANAGE_REFERENCE">
							<Button
								size="sm"
								onClick={() => onOpenEditor({ mode: 'create' })}
							>
								<Plus aria-hidden /> Nouvel article
							</Button>
						</RequirePermission>
					)}
				</EmptyState>
			) : (
				<CatalogTable
					rows={rows}
					extrasOf={extrasOf}
					usageOf={usageOf}
					canManage={canManage}
					onEdit={(row) => onOpenEditor({ mode: 'edit', row })}
					onDuplicate={(row) => onOpenEditor({ mode: 'duplicate', row })}
					onToggleArchive={onToggleArchive}
					onDelete={onDeleteRequest}
				/>
			)}
			<CatalogFormDialog
				editor={editor}
				open={editing}
				initial={formInitial}
				categories={categories}
				customUnits={customUnits}
				isPending={isSaving}
				error={saveError}
				sameReference={sameReference}
				onClose={onCloseEditor}
				onCreateCustomUnit={onCreateCustomUnit}
				isCreatingCustomUnit={isCreatingCustomUnit}
				onSubmit={onSubmit}
				productExtras={productExtras}
			/>
			<ConfirmDialog
				open={deleting !== null}
				onOpenChange={(open) => {
					if (!open) onDeleteRequest(null)
				}}
				title={deleting ? `Supprimer ${deleting.name} ?` : ''}
				description="L’article est retiré du catalogue. Les devis et factures existants ne changent pas."
				confirmLabel="Supprimer"
				destructive
				onConfirm={() => {
					if (deleting) onDeleteConfirm(deleting)
				}}
			/>
		</PageShell>
	)
}
