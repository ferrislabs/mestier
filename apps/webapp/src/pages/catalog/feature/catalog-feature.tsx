import { useForm } from '@tanstack/react-form'
import { AlertCircle } from 'lucide-react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { ProductCatalogFormValues } from '#/hooks/use-catalog-items'
import {
	useCreateProduct,
	useCreateServiceRate,
	useDeleteProduct,
	useDeleteServiceRate,
	useReferenceCatalog,
	useUpdateProduct,
	useUpdateServiceRate,
} from '#/hooks/use-reference-catalog'
import { CatalogUI } from '#/pages/catalog/ui/catalog-ui'
import type { ServiceRateFormValues } from '#/pages/settings/types'

export function CatalogFeature() {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le catalogue nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<CatalogWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
		/>
	)
}

function CatalogWorkspace({
	organizationId,
	organizationName,
}: {
	organizationId: string
	organizationName: string
}) {
	const catalog = useReferenceCatalog(organizationId, {
		employees: false,
		equipment: false,
	})
	const createServiceRate = useCreateServiceRate(organizationId)
	const updateServiceRate = useUpdateServiceRate()
	const deleteServiceRate = useDeleteServiceRate()
	const createProduct = useCreateProduct(organizationId)
	const updateProduct = useUpdateProduct()
	const deleteProduct = useDeleteProduct()

	const serviceRateForm = useForm({
		defaultValues: {
			label: '',
			unit: 'HOUR',
			rate: '',
			vatRate: '20',
		} satisfies ServiceRateFormValues,
		onSubmit: async ({ value }) => {
			await createServiceRate.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					label: value.label.trim(),
					unit: value.unit,
					rate_cents: eurosToCents(value.rate),
					vat_rate: value.vatRate.replace(',', '.').trim(),
				},
			})
			serviceRateForm.reset()
		},
	})

	const productForm = useForm({
		defaultValues: {
			name: '',
			sku: '',
			unit: 'M2',
			unitPrice: '',
			vatRate: '20',
			description: '',
		} satisfies ProductCatalogFormValues,
		onSubmit: async ({ value }) => {
			await createProduct.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: value.name.trim(),
					sku: value.sku.trim() || null,
					unit: value.unit,
					unit_price_cents: eurosToCents(value.unitPrice),
					vat_rate: value.vatRate.replace(',', '.').trim(),
					description: value.description.trim() || null,
				},
			})
			productForm.reset()
		},
	})

	const isLoading = catalog.serviceRates.isLoading || catalog.products.isLoading
	const error =
		catalog.serviceRates.error ??
		catalog.products.error ??
		createServiceRate.error ??
		updateServiceRate.error ??
		deleteServiceRate.error ??
		createProduct.error ??
		updateProduct.error ??
		deleteProduct.error

	return (
		<serviceRateForm.Subscribe selector={(state) => state.values}>
			{(serviceRateValues) => (
				<productForm.Subscribe selector={(state) => state.values}>
					{(productValues) => (
						<CatalogUI
							organizationName={organizationName}
							isLoading={isLoading}
							error={error?.message ?? null}
							serviceRates={catalog.serviceRates.data?.data ?? []}
							products={catalog.products.data?.data ?? []}
							serviceRateForm={{
								values: serviceRateValues,
								isPending: createServiceRate.isPending,
								onChange: (patch) => {
									for (const key of Object.keys(
										patch,
									) as (keyof ServiceRateFormValues)[]) {
										serviceRateForm.setFieldValue(key, patch[key] as never)
									}
								},
								onSubmit: () => serviceRateForm.handleSubmit(),
							}}
							productForm={{
								values: productValues,
								isPending: createProduct.isPending,
								onChange: (patch) => {
									for (const key of Object.keys(
										patch,
									) as (keyof ProductCatalogFormValues)[]) {
										productForm.setFieldValue(key, patch[key] as never)
									}
								},
								onSubmit: () => productForm.handleSubmit(),
							}}
							onUpdateServiceRate={(serviceRate, values) =>
								updateServiceRate.mutateAsync({
									path: { service_rate_id: serviceRate.id },
									body: {
										label: values.label.trim(),
										unit: values.unit,
										rate_cents: eurosToCents(values.rate),
										vat_rate: values.vatRate.replace(',', '.').trim(),
									},
								})
							}
							onDeleteServiceRate={(serviceRate) =>
								deleteServiceRate.mutateAsync({
									path: { service_rate_id: serviceRate.id },
								})
							}
							onUpdateProduct={(product, values) =>
								updateProduct.mutateAsync({
									path: { product_id: product.id },
									body: {
										name: values.name.trim(),
										sku: values.sku.trim() || null,
										unit: values.unit,
										unit_price_cents: eurosToCents(values.unitPrice),
										vat_rate: values.vatRate.replace(',', '.').trim(),
										description: values.description.trim() || null,
									},
								})
							}
							onDeleteProduct={(product) =>
								deleteProduct.mutateAsync({
									path: { product_id: product.id },
								})
							}
						/>
					)}
				</productForm.Subscribe>
			)}
		</serviceRateForm.Subscribe>
	)
}

function eurosToCents(value: string): number {
	const normalized = value.replace(',', '.').trim()
	const parsed = Number.parseFloat(normalized)
	if (!Number.isFinite(parsed)) return 0
	return Math.round(parsed * 100)
}
