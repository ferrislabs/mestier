import { useMemo, useState } from 'react'
import { Label } from '#/components/ui/label'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useBillingSettings } from '#/hooks/use-billing-settings'
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
	apiFrequencyOf,
	bpOfVatRate,
	type CatalogFilters,
	type CatalogForm,
	type CatalogRow,
	catalogRows,
	catalogSummary,
	categoryOptions,
	duplicateOf,
	EMPTY_CATALOG_EXTRAS,
	emptyForm,
	filterCatalog,
	formOf,
	hasSameReference,
	NEVER_USED,
} from '#/pages/catalog/catalog-model'
import { ProductTechnicalSheetField } from '#/pages/catalog/feature/product-technical-sheet-field'
import { buildProductRequest } from '#/pages/catalog/lib/product-request'
import type { CatalogEditor } from '#/pages/catalog/ui/catalog-form-dialog'
import { CatalogUI } from '#/pages/catalog/ui/catalog-ui'
import {
	billingOf,
	DEFAULT_BILLING,
} from '#/pages/crm-settings/lib/settings-api'
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
	const billing = useBillingSettings(organizationId)
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
	const rowOf = (itemId: string) => rows.find((row) => row.id === itemId)
	const extrasOf = (itemId: string) =>
		rowOf(itemId)?.extras ?? EMPTY_CATALOG_EXTRAS
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
	const defaultVat = (
		billing.data ? billingOf(billing.data.data) : DEFAULT_BILLING
	).defaultVatRate

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
		const catalog = {
			category: form.category.trim(),
			billing_frequency: apiFrequencyOf(form.frequency),
			cost_cents: costCents,
			archived:
				editor.mode === 'edit' ? extrasOf(editor.row.id).archived : false,
		}
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
				...catalog,
			}
			if (editor.mode === 'edit' && editor.row.product) {
				await updateProduct.mutateAsync({
					path: { product_id: editor.row.id },
					body,
				})
			} else {
				await createProduct.mutateAsync({
					path: { organization_id: organizationId },
					body,
				})
			}
		} else {
			const body = {
				label: form.name.trim(),
				unit: form.unit,
				rate_cents: priceCents,
				description: form.description.trim() || null,
				default_vat_rate_bp: vat,
				...catalog,
			}
			if (editor.mode === 'edit' && editor.row.serviceRate) {
				await updateServiceRate.mutateAsync({
					path: { service_rate_id: editor.row.id },
					body,
				})
			} else {
				await createServiceRate.mutateAsync({
					path: { organization_id: organizationId },
					body,
				})
			}
		}
		setEditing(false)
	}

	async function toggleArchive(row: CatalogRow) {
		const archived = !row.extras.archived
		const catalog = {
			category: row.extras.category,
			billing_frequency: apiFrequencyOf(row.extras.frequency),
			cost_cents: row.extras.cost,
			archived,
		}
		if (row.product)
			await updateProduct.mutateAsync({
				path: { product_id: row.id },
				body: {
					name: row.product.name,
					sku: row.product.sku ?? null,
					unit: row.product.unit,
					unit_price_cents: row.product.unit_price_cents,
					default_vat_rate_bp: row.product.default_vat_rate_bp ?? null,
					description: row.product.description ?? null,
					photo_keys: row.product.photo_keys,
					supplier_id: row.product.supplier_id ?? null,
					manufacturer: row.product.manufacturer ?? null,
					technical_specs: row.product.technical_specs ?? null,
					datasheet_keys: row.product.datasheet_keys,
					...catalog,
				},
			})
		else if (row.serviceRate)
			await updateServiceRate.mutateAsync({
				path: { service_rate_id: row.id },
				body: {
					label: row.serviceRate.label,
					unit: row.serviceRate.unit,
					rate_cents: row.serviceRate.rate_cents,
					default_vat_rate_bp: row.serviceRate.default_vat_rate_bp ?? null,
					description: row.serviceRate.description ?? null,
					...catalog,
				},
			})
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
			usageOf={(itemId) => rowOf(itemId)?.usage ?? NEVER_USED}
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
			onToggleArchive={(row) => void toggleArchive(row)}
			deleting={deleting}
			onDeleteRequest={setDeleting}
			onDeleteConfirm={(row) => void remove(row)}
		/>
	)
}
