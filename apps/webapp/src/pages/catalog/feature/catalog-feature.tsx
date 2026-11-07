import { useMemo, useState } from 'react'
import { Label } from '#/components/ui/label'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useUploadFile } from '#/hooks/use-customers'
import { useFileUrls } from '#/hooks/use-file-url'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useCreateCustomUnit,
	useCreateProduct,
	useCreateServiceRate,
	useDeleteProduct,
	useDeleteServiceRate,
	useReferenceCatalog,
	useUpdateProduct,
	useUpdateServiceRate,
} from '#/hooks/use-reference-catalog'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	bpOfVatRate,
	type CatalogFilters,
	type CatalogForm,
	type CatalogRow,
	catalogRows,
	catalogSummary,
	categoryOptions,
	duplicateOf,
	emptyForm,
	filterCatalog,
	formOf,
	hasSameReference,
	usageOf,
} from '#/pages/catalog/catalog-model'
import {
	EMPTY_CATALOG_EXTRAS,
	useCatalogExtrasMap,
	useSaveCatalogExtras,
} from '#/pages/catalog/feature/mocks/catalog-extras'
import { useCatalogUsageMap } from '#/pages/catalog/feature/mocks/catalog-usage'
import { ProductTechnicalSheetField } from '#/pages/catalog/feature/product-technical-sheet-field'
import { buildProductRequest } from '#/pages/catalog/lib/product-request'
import type { CatalogEditor } from '#/pages/catalog/ui/catalog-form-dialog'
import { CatalogUI } from '#/pages/catalog/ui/catalog-ui'
import { useCrmSettings } from '#/pages/crm-settings/feature/mocks/settings'
import { parseAmount } from '#/pages/customers/lib/format'
import { PhotoStrip } from '#/pages/quotes/ui/photo-strip'

interface CatalogFeatureProps {
	filters: CatalogFilters
	onFiltersChange: (filters: CatalogFilters) => void
}

export function CatalogFeature(props: CatalogFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	return (
		<CatalogWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

function ProductPhotoField({
	photoKeys,
	isUploading,
	onUpload,
	onRemove,
}: {
	photoKeys: string[]
	isUploading?: boolean
	onUpload: (file: File) => void
	onRemove: (key: string) => void
}) {
	const photos = useFileUrls(photoKeys)
	return (
		<div className="flex flex-col gap-2">
			<Label>Photos</Label>
			<PhotoStrip
				photos={photos}
				isUploading={isUploading}
				onAdd={onUpload}
				onRemove={onRemove}
			/>
		</div>
	)
}

function CatalogWorkspace({
	organizationId,
	filters,
	onFiltersChange,
}: CatalogFeatureProps & { organizationId: string }) {
	const catalog = useReferenceCatalog(organizationId, {
		members: false,
		employeeProfiles: false,
		equipment: false,
	})
	const createServiceRate = useCreateServiceRate(organizationId)
	const updateServiceRate = useUpdateServiceRate()
	const deleteServiceRate = useDeleteServiceRate()
	const createProduct = useCreateProduct(organizationId)
	const updateProduct = useUpdateProduct()
	const deleteProduct = useDeleteProduct()
	const uploadFile = useUploadFile()
	const createCustomUnit = useCreateCustomUnit(organizationId)
	const canManage = useHasPermission('MANAGE_REFERENCE')
	const extras = useCatalogExtrasMap()
	const saveExtras = useSaveCatalogExtras()
	const usage = useCatalogUsageMap()
	const settings = useCrmSettings()
	const [query, setQuery] = useState(filters.query)
	const [editor, setEditor] = useState<CatalogEditor>({ mode: 'create' })
	const [editing, setEditing] = useState(false)
	const [deleting, setDeleting] = useState<CatalogRow | null>(null)

	const products = catalog.products.data?.data
	const serviceRates = catalog.serviceRates.data?.data
	const rows = useMemo(
		() => catalogRows(products ?? [], serviceRates ?? []),
		[products, serviceRates],
	)
	const extrasOf = (itemId: string) => extras[itemId] ?? EMPTY_CATALOG_EXTRAS
	const current: CatalogFilters = { ...filters, query }
	const visible = filterCatalog(rows, extrasOf, current)
	const categories = categoryOptions(
		rows
			.filter((row) => !extrasOf(row.id).archived)
			.map((row) => extrasOf(row.id).category),
		current.category,
	)
	const customUnits = (catalog.customUnits.data?.data ?? []).map(
		(customUnit) => customUnit.code,
	)
	const defaultVat = settings.data.settings.defaultVatRate

	const initial: CatalogForm =
		editor.mode === 'edit'
			? formOf(editor.row, extrasOf(editor.row.id))
			: editor.mode === 'duplicate'
				? duplicateOf(editor.row, extrasOf(editor.row.id))
				: emptyForm(defaultVat)

	const saveError =
		mutationErrorMessage(createProduct.error) ??
		mutationErrorMessage(updateProduct.error) ??
		mutationErrorMessage(createServiceRate.error) ??
		mutationErrorMessage(updateServiceRate.error)

	async function submit(form: CatalogForm) {
		const priceCents = parseAmount(form.price) ?? 0
		const costCents = form.cost.trim() === '' ? null : parseAmount(form.cost)
		const vat = bpOfVatRate(form.vatRate)
		const price = (priceCents / 100).toFixed(2)
		let itemId: string
		if (form.kind === 'product') {
			const body = {
				...buildProductRequest({
					name: form.name,
					sku: form.reference,
					unit: form.unit,
					unitPrice: price,
					description: form.description,
					photoKeys: form.photoKeys,
					supplierId: form.supplierId,
					manufacturer: form.manufacturer,
					technicalSpecs: form.technicalSpecs,
					datasheetKeys: form.datasheetKeys,
				}),
				default_vat_rate_bp: vat,
			}
			if (editor.mode === 'edit' && editor.row.product) {
				itemId = editor.row.id
				await updateProduct.mutateAsync({
					path: { product_id: itemId },
					body,
				})
			} else {
				const created = await createProduct.mutateAsync({
					path: { organization_id: organizationId },
					body,
				})
				itemId = created.data.id
			}
		} else {
			const body = {
				label: form.name.trim(),
				unit: form.unit,
				rate_cents: priceCents,
				description: form.description.trim() || null,
				default_vat_rate_bp: vat,
			}
			if (editor.mode === 'edit' && editor.row.serviceRate) {
				itemId = editor.row.id
				await updateServiceRate.mutateAsync({
					path: { service_rate_id: itemId },
					body,
				})
			} else {
				const created = await createServiceRate.mutateAsync({
					path: { organization_id: organizationId },
					body,
				})
				itemId = created.data.id
			}
		}
		await saveExtras.mutateAsync(itemId, {
			category: form.category.trim(),
			cost: costCents,
			frequency: form.frequency,
		})
		setEditing(false)
	}

	async function remove(row: CatalogRow) {
		if (row.product)
			await deleteProduct.mutateAsync({ path: { product_id: row.id } })
		else
			await deleteServiceRate.mutateAsync({
				path: { service_rate_id: row.id },
			})
		setDeleting(null)
	}

	return (
		<CatalogUI
			summary={catalogSummary(rows, extrasOf)}
			totalRows={rows.length}
			rows={visible}
			filters={current}
			categories={categories}
			onFiltersChange={(next) => {
				setQuery(next.query)
				onFiltersChange(next)
			}}
			extrasOf={extrasOf}
			usageOf={(itemId) => usageOf(usage, itemId)}
			canManage={canManage}
			isLoading={catalog.serviceRates.isLoading || catalog.products.isLoading}
			error={
				mutationErrorMessage(catalog.serviceRates.error) ??
				mutationErrorMessage(catalog.products.error) ??
				mutationErrorMessage(deleteProduct.error) ??
				mutationErrorMessage(deleteServiceRate.error)
			}
			editor={editor}
			editing={editing}
			formInitial={initial}
			customUnits={customUnits}
			isSaving={
				createProduct.isPending ||
				updateProduct.isPending ||
				createServiceRate.isPending ||
				updateServiceRate.isPending
			}
			saveError={saveError}
			sameReference={(reference) =>
				hasSameReference(
					rows,
					reference,
					editor.mode === 'edit' ? editor.row.id : null,
				)
			}
			onOpenEditor={(next) => {
				setEditor(next)
				setEditing(true)
			}}
			onCloseEditor={() => setEditing(false)}
			onCreateCustomUnit={(code) =>
				createCustomUnit.mutate({
					path: { organization_id: organizationId },
					body: { code },
				})
			}
			isCreatingCustomUnit={createCustomUnit.isPending}
			onSubmit={submit}
			productExtras={(form, change) => (
				<>
					<ProductPhotoField
						photoKeys={form.photoKeys}
						isUploading={uploadFile.isPending}
						onUpload={async (file) => {
							const uploaded = await uploadFile.mutateAsync(file)
							change({ photoKeys: [...form.photoKeys, uploaded.data.key] })
						}}
						onRemove={(key) =>
							change({
								photoKeys: form.photoKeys.filter(
									(photoKey) => photoKey !== key,
								),
							})
						}
					/>
					<ProductTechnicalSheetField
						values={form}
						isUploading={uploadFile.isPending}
						onChange={(patch) => change(patch)}
						onUpload={async (file) => {
							const uploaded = await uploadFile.mutateAsync(file)
							change({
								datasheetKeys: [...form.datasheetKeys, uploaded.data.key],
							})
						}}
						onRemove={(key) =>
							change({
								datasheetKeys: form.datasheetKeys.filter(
									(datasheetKey) => datasheetKey !== key,
								),
							})
						}
					/>
				</>
			)}
			onToggleArchive={(row) =>
				void saveExtras.mutateAsync(row.id, {
					archived: !extrasOf(row.id).archived,
				})
			}
			deleting={deleting}
			onDeleteRequest={setDeleting}
			onDeleteConfirm={(row) => void remove(row)}
		/>
	)
}
