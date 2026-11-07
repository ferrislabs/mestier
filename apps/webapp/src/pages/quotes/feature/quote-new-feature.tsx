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
import type { Organization } from '#/hooks/use-organizations'
import { useCreateQuote } from '#/hooks/use-quotes'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import {
	duplicateItem,
	moveItem,
	newLineId,
} from '#/pages/quotes/lib/doc-lines'
import { defaultValidUntil } from '#/pages/quotes/lib/quote-draft'
import { termsBody } from '#/pages/quotes/lib/quote-form-terms'
import {
	copyQuoteLine,
	lineFromCatalogItem,
	quoteLineBodies,
} from '#/pages/quotes/lib/quote-lines'
import { toIsoDate } from '#/pages/quotes/lib/quote-status'
import {
	readQuoteClientParam,
	readQuoteDealParam,
} from '#/pages/quotes/quote-list-url-state'
import {
	emptyQuoteLine,
	type QuoteFormValues,
	type QuoteLineFormValues,
} from '#/pages/quotes/types'
import { QuoteNewUI } from '#/pages/quotes/ui/quote-new-ui'
import type { QuoteOwner } from '#/pages/quotes/ui/quote-terms-fields'

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
			organization={activeOrganization}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			vatEnabled={activeOrganization.vat_status?.type === 'subject'}
		/>
	)
}

function QuoteNewWorkspace({
	organization,
	organizationId,
	organizationSlug,
	vatEnabled,
}: {
	organization: Organization
	organizationId: string
	organizationSlug: string
	vatEnabled: boolean
}) {
	const navigate = useNavigate()
	const catalog = useReferenceCatalog(organizationId, {
		members: true,
		employeeProfiles: false,
		equipment: false,
	})
	const customers = useCustomers(organizationId)
	const catalogItems = useCatalogItems(
		catalog.serviceRates.data?.data,
		catalog.products.data?.data,
	)
	const createQuote = useCreateQuote(organizationId)
	const uploadFile = useUploadFile()
	const owners = (catalog.members.data?.data ?? []).map((member) => ({
		id: member.id,
		name: member.display_name,
	}))

	const form = useForm({
		defaultValues: {
			title: '',
			customerId: readQuoteClientParam() ?? '',
			customerContextId: '',
			lines: [emptyQuoteLine()],
			ownerId: null,
			validUntil: defaultValidUntil(toIsoDate(new Date())),
			depositPercent: '',
			notes: '',
		} as QuoteFormValues,
		onSubmit: async ({ value }) => {
			const quote = await createQuote.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					title: value.title.trim(),
					customer_id: value.customerId,
					customer_context_id: value.customerContextId,
					deal_id: readQuoteDealParam() ?? null,
					...termsBody(value),
					lines: quoteLineBodies(value.lines),
				},
			})

			// Straight to the quote that was just written, not back to the list.
			// The next thing an artisan does is send it or export it, and both
			// live on that page. The modal used to reset the form in place and
			// show a success banner underneath itself, where nobody saw it.
			await navigate({
				to: buildOrgPath(organizationSlug, '/crm/devis/$quoteId'),
				params: { quoteId: quote.data.id },
			})
		},
	})

	return (
		<form.Subscribe selector={(state) => state.values}>
			{(values) => (
				<QuoteNewForm
					organization={organization}
					organizationSlug={organizationSlug}
					values={values}
					form={form}
					customers={customers.data?.data ?? []}
					catalogItems={catalogItems}
					customUnits={(catalog.customUnits.data?.data ?? []).map(
						(customUnit) => customUnit.code,
					)}
					vatEnabled={vatEnabled}
					owners={owners}
					isCreating={createQuote.isPending}
					isUploading={uploadFile.isPending}
					error={
						mutationErrorMessage(createQuote.error) ??
						mutationErrorMessage(uploadFile.error) ??
						mutationErrorMessage(customers.error) ??
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
	organization: Organization
	organizationSlug: string
	values: QuoteFormValues
	form: QuoteFormApi
	customers: Customer[]
	catalogItems: CatalogItem[]
	customUnits: string[]
	vatEnabled: boolean
	owners: readonly QuoteOwner[]
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
	organization,
	organizationSlug,
	values,
	form,
	customers,
	catalogItems,
	customUnits,
	vatEnabled,
	owners,
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
		if (patch.ownerId !== undefined)
			form.setFieldValue('ownerId', patch.ownerId)
		if (patch.validUntil !== undefined) {
			form.setFieldValue('validUntil', patch.validUntil)
		}
		if (patch.depositPercent !== undefined) {
			form.setFieldValue('depositPercent', patch.depositPercent)
		}
		if (patch.notes !== undefined) form.setFieldValue('notes', patch.notes)
	}

	const updateLine = (index: number, patch: Partial<QuoteLineFormValues>) => {
		const lines = [...form.state.values.lines]
		const current = lines[index]
		if (!current) return
		lines[index] = { ...current, ...patch }
		form.setFieldValue('lines', lines)
	}

	const appendLine = (line: QuoteLineFormValues) => {
		form.setFieldValue('lines', [...form.state.values.lines, line])
		return line.clientId
	}

	return (
		<QuoteNewUI
			organization={organization}
			organizationSlug={organizationSlug}
			values={values}
			customers={customers}
			customerContexts={customerContexts.data?.data ?? []}
			catalogItems={catalogItems}
			customUnits={customUnits}
			owners={owners}
			photoUrls={photoUrls}
			error={error}
			isCreating={isCreating}
			isUploading={isUploading}
			isCustomerContextsLoading={customerContexts.isLoading}
			vatEnabled={vatEnabled}
			onChange={updateValues}
			onLineChange={updateLine}
			onAddLine={() =>
				// Date-stamped: a counter derived from the length collides after a
				// removal and React would then see two lines with the same key.
				appendLine(emptyQuoteLine(newLineId(form.state.values.lines.length)))
			}
			onAddCatalogItem={(item) =>
				appendLine(
					lineFromCatalogItem(item, newLineId(form.state.values.lines.length)),
				)
			}
			onMoveLine={(index, delta) =>
				form.setFieldValue(
					'lines',
					moveItem(form.state.values.lines, index, delta),
				)
			}
			onDuplicateLine={(index) => {
				const clientId = newLineId(form.state.values.lines.length)
				form.setFieldValue(
					'lines',
					duplicateItem(form.state.values.lines, index, (line) =>
						copyQuoteLine(line, clientId),
					),
				)
				return clientId
			}}
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
