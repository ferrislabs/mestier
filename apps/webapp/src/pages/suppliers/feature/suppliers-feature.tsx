import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	type Supplier,
	useCreateSupplier,
	useSuppliers,
	useUpdateSupplier,
} from '#/hooks/use-suppliers'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	buildSupplierRequest,
	EMPTY_SUPPLIER_FORM,
	supplierToFormValues,
} from '#/pages/suppliers/types'
import { SupplierFormDialog } from '#/pages/suppliers/ui/supplier-form-dialog'
import { SuppliersList } from '#/pages/suppliers/ui/suppliers-list'

export function SuppliersFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<SuppliersWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
		/>
	)
}

type Draft = { supplier: Supplier | null }

function SuppliersWorkspace({
	organizationId,
	organizationName,
}: {
	organizationId: string
	organizationName: string
}) {
	const [draft, setDraft] = useState<Draft | null>(null)
	const canManage = useHasPermission('MANAGE_REFERENCE')

	const suppliers = useSuppliers(organizationId)
	const createSupplier = useCreateSupplier()
	const updateSupplier = useUpdateSupplier()

	const rows = suppliers.data?.data ?? []
	const mutationError = createSupplier.error ?? updateSupplier.error
	const listError = suppliers.error

	return (
		<>
			<SuppliersList
				organizationName={organizationName}
				suppliers={rows}
				isLoading={suppliers.isLoading}
				error={listError ? mutationErrorMessage(listError) : null}
				canManage={canManage}
				onCreate={() => {
					createSupplier.reset()
					setDraft({ supplier: null })
				}}
				onEdit={(supplier) => {
					updateSupplier.reset()
					setDraft({ supplier })
				}}
			/>

			{draft ? (
				<SupplierFormDialog
					key={draft.supplier?.id ?? 'new'}
					open
					editingName={draft.supplier?.name ?? null}
					initialValues={
						draft.supplier
							? supplierToFormValues(draft.supplier)
							: EMPTY_SUPPLIER_FORM
					}
					isPending={createSupplier.isPending || updateSupplier.isPending}
					error={mutationError ? mutationErrorMessage(mutationError) : null}
					onOpenChange={(open) => {
						if (!open) setDraft(null)
					}}
					onSubmit={async (values) => {
						const body = buildSupplierRequest(values)
						if (draft.supplier) {
							await updateSupplier.mutateAsync({
								path: {
									organization_id: organizationId,
									supplier_id: draft.supplier.id,
								},
								body,
							})
						} else {
							await createSupplier.mutateAsync({
								path: { organization_id: organizationId },
								body,
							})
						}
						setDraft(null)
					}}
				/>
			) : null}
		</>
	)
}
