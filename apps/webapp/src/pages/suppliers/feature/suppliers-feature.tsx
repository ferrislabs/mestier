import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useCreateSupplier,
	useSuppliers,
	useUpdateSupplier,
} from '#/hooks/use-suppliers'
import { mutationErrorMessage } from '#/lib/api-error'
import { useNotice } from '#/pages/customers/feature/use-notice'
import { todayIso } from '#/pages/customers/lib/format'
import {
	EMPTY_SUPPLIER_EXTRAS,
	useSaveSupplierExtras,
	useSupplierExtrasMap,
} from '#/pages/suppliers/feature/mocks/supplier-extras'
import {
	buildSupplierRequest,
	emptySupplierValues,
	extrasOfValues,
	supplierValuesOf,
} from '#/pages/suppliers/lib/supplier-form'
import {
	type SupplierRow,
	supplierRows,
	supplierTotals,
} from '#/pages/suppliers/lib/supplier-model'
import type { SupplierFormValues } from '#/pages/suppliers/types'
import { SupplierFormDialog } from '#/pages/suppliers/ui/supplier-form-dialog'
import { SuppliersList } from '#/pages/suppliers/ui/suppliers-list'
import { useLedger } from '#/pages/treasury/feature/mocks/ledger'

interface SuppliersFeatureProps {
	archived?: boolean
	onArchivedChange?: (archived: boolean) => void
}

export function SuppliersFeature(props: SuppliersFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<SuppliersWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

type Draft = { row: SupplierRow | null }

function SuppliersWorkspace({
	organizationId,
	archived: archivedProp,
	onArchivedChange,
}: SuppliersFeatureProps & { organizationId: string }) {
	const [draft, setDraft] = useState<Draft | null>(null)
	const [query, setQuery] = useState('')
	const [archivedLocal, setArchivedLocal] = useState(false)
	const archived = archivedProp ?? archivedLocal
	const notice = useNotice()
	const canManage = useHasPermission('MANAGE_REFERENCE')

	const suppliers = useSuppliers(organizationId)
	const createSupplier = useCreateSupplier()
	const updateSupplier = useUpdateSupplier()
	const extras = useSupplierExtrasMap()
	const saveExtras = useSaveSupplierExtras()
	const ledger = useLedger()

	const all = suppliers.data?.data
	const today = todayIso()
	const rows = useMemo(
		() =>
			supplierRows({
				suppliers: all ?? [],
				extras,
				expenses: ledger.data.expenses,
				categories: ledger.data.categories,
				today,
				filters: { archived, query },
			}),
		[all, extras, ledger.data, today, archived, query],
	)
	const totals = useMemo(() => supplierTotals(rows), [rows])
	const mutationError = createSupplier.error ?? updateSupplier.error
	const listError = suppliers.error

	async function save(values: SupplierFormValues) {
		const body = buildSupplierRequest(values)
		const patch = extrasOfValues(values)
		if (draft?.row) {
			const supplierId = draft.row.supplier.id
			await updateSupplier.mutateAsync({
				path: { organization_id: organizationId, supplier_id: supplierId },
				body,
			})
			saveExtras.mutate(supplierId, patch)
			notice.show('Fournisseur mis à jour.')
		} else {
			const created = await createSupplier.mutateAsync({
				path: { organization_id: organizationId },
				body,
			})
			saveExtras.mutate(created.data.id, patch)
			notice.show('Fournisseur créé.')
		}
		setDraft(null)
	}

	function toggleArchive(row: SupplierRow) {
		const archive = !row.extras.archived
		saveExtras.mutate(row.supplier.id, { archived: archive })
		notice.show(archive ? 'Fournisseur archivé.' : 'Fournisseur réactivé.')
		setDraft(null)
	}

	return (
		<>
			<SuppliersList
				rows={rows}
				totals={totals}
				totalSuppliers={all?.length ?? 0}
				query={query}
				onQueryChange={setQuery}
				archived={archived}
				onArchivedChange={onArchivedChange ?? setArchivedLocal}
				isLoading={suppliers.isLoading}
				error={listError ? mutationErrorMessage(listError) : null}
				canManage={canManage}
				notice={notice.message}
				onDismissNotice={notice.clear}
				onCreate={() => {
					createSupplier.reset()
					setDraft({ row: null })
				}}
				onEdit={(row) => {
					updateSupplier.reset()
					setDraft({ row })
				}}
			/>

			{draft ? (
				<SupplierFormDialog
					key={draft.row?.supplier.id ?? 'new'}
					open
					editingName={draft.row?.supplier.name ?? null}
					initialValues={
						draft.row
							? supplierValuesOf(
									draft.row.supplier,
									extras[draft.row.supplier.id] ?? EMPTY_SUPPLIER_EXTRAS,
								)
							: emptySupplierValues()
					}
					categories={ledger.data.categories}
					archived={draft.row ? draft.row.extras.archived : null}
					toPay={draft.row?.toPay ?? 0}
					isPending={createSupplier.isPending || updateSupplier.isPending}
					error={mutationError ? mutationErrorMessage(mutationError) : null}
					onOpenChange={(open) => {
						if (!open) setDraft(null)
					}}
					onSubmit={(values) => void save(values).catch(() => undefined)}
					onToggleArchive={() => {
						if (draft.row) toggleArchive(draft.row)
					}}
				/>
			) : null}
		</>
	)
}
