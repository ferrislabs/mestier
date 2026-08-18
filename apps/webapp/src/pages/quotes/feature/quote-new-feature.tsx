import { useForm } from '@tanstack/react-form'
import { useNavigate } from '@tanstack/react-router'
import { AlertCircle } from 'lucide-react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { type CatalogItem, useCatalogItems } from '#/hooks/use-catalog-items'
import {
	type Customer,
	useCustomerContexts,
	useCustomers,
	useUploadFile,
} from '#/hooks/use-customers'
import { useFileUrls } from '#/hooks/use-file-url'
import { useCreateQuote } from '#/hooks/use-quotes'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { buildOrgPath } from '#/modules/org-path'
import {
	centsToEuros,
	emptyQuoteLine,
	eurosToCents,
	type QuoteFormValues,
	type QuoteLineFormValues,
} from '#/pages/quotes/types'
import { QuoteNewUI } from '#/pages/quotes/ui/quote-new-ui'

export function QuoteNewFeature() {
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
						La création de devis nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<QuoteNewWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

function QuoteNewWorkspace({
	organizationId,
	organizationSlug,
}: {
	organizationId: string
	organizationSlug: string
}) {
	const navigate = useNavigate()
	const catalog = useReferenceCatalog(organizationId, {
		members: false,
		employeeProfiles: false,
		equipment: false,
	})
	const customers = useCustomers(organizationId)
	const catalogItems = useCatalogItems(
		catalog.serviceRates.data?.data ?? [],
		catalog.products.data?.data ?? [],
	)
	const createQuote = useCreateQuote(organizationId)
	const uploadFile = useUploadFile()

	const form = useForm({
		defaultValues: {
			title: '',
			customerId: '',
			customerContextId: '',
			lines: [emptyQuoteLine()],
		} satisfies QuoteFormValues,
		onSubmit: async ({ value }) => {
			const quote = await createQuote.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					title: value.title.trim(),
					customer_id: value.customerId,
					customer_context_id: value.customerContextId,
					lines: value.lines.map((line) => ({
						service_rate_id: line.serviceRateId || null,
						label: line.label.trim(),
						quantity: line.quantity.replace(',', '.').trim(),
						unit: line.unit,
						unit_price_cents: eurosToCents(line.unitPrice),
						notes: line.notes.trim() || null,
						photo_keys: line.photoKeys,
					})),
				},
			})

			// Straight to the quote that was just written, not back to the list.
			// The next thing an artisan does is send it or export it, and both
			// live on that page. The modal used to reset the form in place and
			// show a success banner underneath itself, where nobody saw it.
			await navigate({
				to: buildOrgPath(organizationSlug, '/crm/quotes/$quoteId'),
				params: { quoteId: quote.data.id },
			})
		},
	})

	return (
		<form.Subscribe selector={(state) => state.values}>
			{(values) => (
				<QuoteNewForm
					organizationSlug={organizationSlug}
					values={values}
					form={form}
					customers={customers.data?.data ?? []}
					catalogItems={catalogItems}
					isCreating={createQuote.isPending}
					isUploading={uploadFile.isPending}
					error={
						createQuote.error?.message ??
						uploadFile.error?.message ??
						customers.error?.message ??
						null
					}
					uploadFile={async (lineIndex, file) => {
						const uploaded = await uploadFile.mutateAsync(file)
						const lines = [...form.state.values.lines]
						const line = lines[lineIndex]
						if (!line) return
						lines[lineIndex] = {
							...line,
							photoKeys: [...line.photoKeys, uploaded.data.key],
						}
						form.setFieldValue('lines', lines)
					}}
				/>
			)}
		</form.Subscribe>
	)
}

interface QuoteFormApi {
	state: { values: QuoteFormValues }
	setFieldValue: (
		field: keyof QuoteFormValues,
		value: QuoteFormValues[keyof QuoteFormValues],
	) => void
	handleSubmit: () => void | Promise<void>
}

interface QuoteNewFormProps {
	organizationSlug: string
	values: QuoteFormValues
	form: QuoteFormApi
	customers: Customer[]
	catalogItems: CatalogItem[]
	isCreating: boolean
	isUploading: boolean
	error: string | null
	uploadFile: (lineIndex: number, file: File) => Promise<void>
}

/**
 * Split from the workspace because the addresses to choose from depend on the
 * customer being edited, and that value only exists inside `form.Subscribe`.
 */
function QuoteNewForm({
	organizationSlug,
	values,
	form,
	customers,
	catalogItems,
	isCreating,
	isUploading,
	error,
	uploadFile,
}: QuoteNewFormProps) {
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)
	const photoPreviews = useFileUrls(
		values.lines.flatMap((line) => line.photoKeys),
	)
	// Resolved here rather than in the line editor: signing is a fetch, and the
	// UI layer of this page does not fetch. One flat list of keys also means a
	// photo shared by two lines is signed once.
	const photoUrls = Object.fromEntries(
		photoPreviews.map((preview) => [preview.key, preview.url]),
	)

	const updateValues = (patch: Partial<QuoteFormValues>) => {
		if (patch.title !== undefined) form.setFieldValue('title', patch.title)
		if (patch.customerId !== undefined) {
			form.setFieldValue('customerId', patch.customerId)
			// The address belongs to the customer, so keeping the old one would
			// bill a new customer at another customer's address.
			form.setFieldValue('customerContextId', '')
		}
		if (patch.customerContextId !== undefined) {
			form.setFieldValue('customerContextId', patch.customerContextId)
		}
		if (patch.lines !== undefined) form.setFieldValue('lines', patch.lines)
	}

	const updateLine = (index: number, patch: Partial<QuoteLineFormValues>) => {
		const lines = [...form.state.values.lines]
		const current = lines[index]
		if (!current) return
		lines[index] = { ...current, ...patch }
		form.setFieldValue('lines', lines)
	}

	const selectCatalogItem = (index: number, catalogItemId: string) => {
		const catalogItem = catalogItems.find((item) => item.id === catalogItemId)
		if (!catalogItem) {
			updateLine(index, {
				catalogItemId: '',
				catalogItemType: 'CUSTOM',
				serviceRateId: '',
			})
			return
		}

		updateLine(index, {
			catalogItemId: catalogItem.id,
			catalogItemType: catalogItem.type,
			serviceRateId: catalogItem.type === 'SERVICE' ? catalogItem.sourceId : '',
			label: catalogItem.label,
			unit: catalogItem.unit,
			unitPrice: centsToEuros(catalogItem.unitPriceCents),
			notes:
				catalogItem.description || form.state.values.lines[index]?.notes || '',
		})
	}

	return (
		<QuoteNewUI
			organizationSlug={organizationSlug}
			values={values}
			customers={customers}
			customerContexts={customerContexts.data?.data ?? []}
			catalogItems={catalogItems}
			photoUrls={photoUrls}
			error={error}
			isCreating={isCreating}
			isUploading={isUploading}
			isCustomerContextsLoading={customerContexts.isLoading}
			onChange={updateValues}
			onLineChange={updateLine}
			onSelectCatalogItem={selectCatalogItem}
			onAddLine={() =>
				form.setFieldValue('lines', [
					...form.state.values.lines,
					// Date-stamped, as the previous form did: a counter derived from
					// the length collides after a removal and React would then
					// see two lines with the same key.
					emptyQuoteLine(
						`line-${Date.now()}-${form.state.values.lines.length}`,
					),
				])
			}
			onRemoveLine={(index) => {
				const next = form.state.values.lines.filter(
					(_line, itemIndex) => itemIndex !== index,
				)
				form.setFieldValue('lines', next.length > 0 ? next : [emptyQuoteLine()])
			}}
			onUploadLinePhoto={uploadFile}
			onSubmit={() => void form.handleSubmit()}
		/>
	)
}
