import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useFileUrls } from '#/hooks/use-file-url'
import { useHasPermission } from '#/hooks/use-permissions'
import { useCreateSupplier, useSuppliers } from '#/hooks/use-suppliers'
import { mutationErrorMessage } from '#/lib/api-error'
import { datasheetName } from '#/pages/catalog/lib/product-request'
import {
	ProductTechnicalSheet,
	type TechnicalSheetValues,
} from '#/pages/catalog/ui/product-technical-sheet'
import {
	buildSupplierRequest,
	EMPTY_SUPPLIER_FORM,
} from '#/pages/suppliers/types'
import { SupplierFormDialog } from '#/pages/suppliers/ui/supplier-form-dialog'

const TYPE_BY_EXTENSION: Record<string, string> = {
	pdf: 'PDF',
	jpg: 'Image',
	jpeg: 'Image',
	png: 'Image',
	webp: 'Image',
}

function datasheetType(key: string): string {
	const extension = key.split('.').at(-1)?.toLowerCase() ?? ''
	return TYPE_BY_EXTENSION[extension] ?? 'Document'
}

export interface ProductTechnicalSheetFieldProps {
	values: TechnicalSheetValues
	isUploading?: boolean
	onChange: (patch: Partial<TechnicalSheetValues>) => void
	onUpload: (file: File) => void
	onRemove: (key: string) => void
}

export function ProductTechnicalSheetField({
	values,
	isUploading,
	onChange,
	onUpload,
	onRemove,
}: ProductTechnicalSheetFieldProps) {
	const { activeOrganization } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_REFERENCE')
	const suppliers = useSuppliers(activeOrganization.id)
	const createSupplier = useCreateSupplier()
	const [dialogOpen, setDialogOpen] = useState(false)
	const previews = useFileUrls(values.datasheetKeys)

	const datasheets = previews.map((preview) => ({
		key: preview.key,
		name: datasheetName(preview.key),
		type: datasheetType(preview.key),
		url: preview.url,
	}))

	return (
		<>
			<ProductTechnicalSheet
				values={values}
				suppliers={suppliers.data?.data ?? []}
				datasheets={datasheets}
				isUploading={isUploading}
				canCreateSupplier={canManage}
				onChange={onChange}
				onUpload={onUpload}
				onRemove={onRemove}
				onCreateSupplier={() => {
					createSupplier.reset()
					setDialogOpen(true)
				}}
			/>
			{dialogOpen ? (
				<SupplierFormDialog
					open
					editingName={null}
					initialValues={EMPTY_SUPPLIER_FORM}
					isPending={createSupplier.isPending}
					error={
						createSupplier.error
							? mutationErrorMessage(createSupplier.error)
							: null
					}
					onOpenChange={(open) => {
						if (!open) setDialogOpen(false)
					}}
					onSubmit={async (supplierValues) => {
						const created = await createSupplier.mutateAsync({
							path: { organization_id: activeOrganization.id },
							body: buildSupplierRequest(supplierValues),
						})
						onChange({ supplierId: created.data.id })
						setDialogOpen(false)
					}}
				/>
			) : null}
		</>
	)
}
