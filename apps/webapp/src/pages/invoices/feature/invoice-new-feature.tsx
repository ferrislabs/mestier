import { useForm } from '@tanstack/react-form'
import { Link, useNavigate } from '@tanstack/react-router'
import { AlertCircle } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	useCustomerContexts,
	useCustomers,
} from '#/hooks/use-customers'
import {
	useCreateInvoice,
	useCreateInvoiceFromQuote,
	useQuoteInvoicing,
} from '#/hooks/use-invoices'
import { type Project, useProjects } from '#/hooks/use-projects'
import { useQuote } from '#/hooks/use-quotes'
import { buildOrgPath } from '#/modules/org-path'
import {
	parseTerms,
	quoteInvoicingView,
	readNewInvoiceSearch,
} from '#/pages/invoices/lib/invoice-new'
import type { NewInvoiceFormValues } from '#/pages/invoices/lib/new-invoice-form'
import {
	emptyInvoiceLine,
	eurosToCents,
	type InvoiceLineFormValues,
	invoiceKindLabel,
} from '#/pages/invoices/types'
import {
	type DepositRow,
	InvoiceFromQuoteUI,
	type QuoteInvoiceRequest,
} from '#/pages/invoices/ui/invoice-from-quote-ui'
import { InvoiceNewUI } from '#/pages/invoices/ui/invoice-new-ui'
import {
	duplicateItem,
	moveItem,
	newLineId,
} from '#/pages/quotes/lib/doc-lines'
import {
	DetailLoading,
	NotFoundState,
} from '#/pages/quotes/ui/doc/detail-shell'

export function InvoiceNewFeature() {
	const { activeOrganization } = useActiveOrganization()
	const search = readNewInvoiceSearch(window.location.search)

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						La création de facture nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	if (search.quoteId) {
		return (
			<InvoiceFromQuoteWorkspace
				key={`${activeOrganization.id}:${search.quoteId}`}
				organizationId={activeOrganization.id}
				organizationSlug={activeOrganization.slug}
				quoteId={search.quoteId}
				preferred={search.mode}
			/>
		)
	}

	return (
		<InvoiceNewWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			vatEnabled={activeOrganization.vat_status?.type === 'subject'}
			clientId={search.clientId}
		/>
	)
}

function InvoiceFromQuoteWorkspace({
	organizationId,
	organizationSlug,
	quoteId,
	preferred,
}: {
	organizationId: string
	organizationSlug: string
	quoteId: string
	preferred: 'acompte' | 'complete' | undefined
}) {
	const navigate = useNavigate()
	const quote = useQuote(quoteId)
	const customers = useCustomers(organizationId)
	const invoicing = useQuoteInvoicing(quoteId)
	const createFromQuote = useCreateInvoiceFromQuote(organizationId)

	if (quote.isLoading || invoicing.isLoading) {
		return <DetailLoading label="Chargement du devis…" />
	}

	const source = quote.data?.data
	if (!source || !invoicing.data) {
		return (
			<NotFoundState
				title="Ce devis est introuvable"
				hint="Il n’existe pas ou l’adresse n’est plus valide."
			>
				<Button variant="outline" size="sm" asChild>
					<Link to={buildOrgPath(organizationSlug, '/crm/devis')}>
						Voir les devis
					</Link>
				</Button>
			</NotFoundState>
		)
	}

	const info = quoteInvoicingView(invoicing.data.data)
	const deposits: DepositRow[] = [
		...info.deposits.issued,
		...info.deposits.drafts,
	].map((link) => ({
		invoiceId: link.invoiceId,
		label: link.draft
			? 'Acompte en brouillon'
			: (link.number ?? invoiceKindLabel('DEPOSIT')),
		draft: link.draft,
		issuedAt: link.issuedAt,
		grossCents: link.grossCents,
	}))
	const customer = customers.data?.data.find(
		(item) => item.id === source.customer_id,
	)

	const create = async (request: QuoteInvoiceRequest) => {
		const created = await createFromQuote.mutateAsync({
			path: { quote_id: source.id },
			body:
				request.kind === 'full'
					? { mode: 'FULL' }
					: { mode: 'DEPOSIT', percent: String(request.percent) },
		})
		await navigate({
			to: buildOrgPath(organizationSlug, '/crm/factures/$invoiceId'),
			params: { invoiceId: created.data.id },
		})
	}

	return (
		<InvoiceFromQuoteUI
			organizationSlug={organizationSlug}
			quote={source}
			customerName={customer ? customer.name.trim() : null}
			info={info}
			deposits={deposits}
			existingInvoiceLabel={info.existingInvoiceNumber}
			preferred={preferred}
			isCreating={createFromQuote.isPending}
			error={createFromQuote.error?.message ?? null}
			onSubmit={(request) => void create(request).catch(() => undefined)}
		/>
	)
}

function InvoiceNewWorkspace({
	organizationId,
	organizationSlug,
	vatEnabled,
	clientId,
}: {
	organizationId: string
	organizationSlug: string
	vatEnabled: boolean
	clientId: string | undefined
}) {
	const navigate = useNavigate()
	const customers = useCustomers(organizationId)
	const projects = useProjects(organizationId, { includeArchived: false })
	const createInvoice = useCreateInvoice(organizationId)

	const form = useForm({
		defaultValues: {
			kind: 'STANDARD',
			projectId: '',
			customerId: clientId ?? '',
			customerContextId: '',
			title: '',
			termsDays: '',
			legalNotice: '',
			notes: '',
			operationNature: '',
			lines: [emptyInvoiceLine()],
		} satisfies NewInvoiceFormValues,
		onSubmit: async ({ value }) => {
			const invoice = await createInvoice.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					kind: value.kind,
					project_id: value.projectId || null,
					customer_id: value.customerId,
					customer_context_id: value.customerContextId,
					title: value.title.trim(),
					payment_terms_days: parseTerms(value.termsDays),
					notes: value.notes.trim() || null,
					operation_nature: value.operationNature || null,
					lines: value.lines.map((line) => ({
						label: line.label.trim(),
						quantity: line.quantity.replace(',', '.').trim(),
						unit_price_cents: eurosToCents(line.unitPrice),
						vat_rate_basis_points:
							line.vatRateBp === '' ? null : Number(line.vatRateBp),
						discount_percent: line.discountPercent,
						product_id: line.productId,
						service_rate_id: line.serviceRateId,
					})),
				},
			})

			// Straight to the invoice just created, same reasoning as
			// `QuoteNewFeature`: the next thing an artisan does — issue it,
			// record a payment, export the PDF — lives on that page.
			await navigate({
				to: buildOrgPath(organizationSlug, '/crm/factures/$invoiceId'),
				params: { invoiceId: invoice.data.id },
			})
		},
	})

	return (
		<form.Subscribe selector={(state) => state.values}>
			{(values) => (
				<InvoiceNewForm
					organizationSlug={organizationSlug}
					values={values}
					form={form}
					customers={customers.data?.data ?? []}
					projects={projects.data?.data ?? []}
					vatEnabled={vatEnabled}
					isCreating={createInvoice.isPending}
					error={
						createInvoice.error?.message ?? customers.error?.message ?? null
					}
				/>
			)}
		</form.Subscribe>
	)
}

interface InvoiceFormApi {
	state: { values: NewInvoiceFormValues }
	setFieldValue: (
		field: keyof NewInvoiceFormValues,
		value: NewInvoiceFormValues[keyof NewInvoiceFormValues],
	) => void
	handleSubmit: () => void | Promise<void>
}

interface InvoiceNewFormProps {
	organizationSlug: string
	values: NewInvoiceFormValues
	form: InvoiceFormApi
	customers: Customer[]
	projects: Project[]
	vatEnabled: boolean
	isCreating: boolean
	error: string | null
}

/**
 * Split from the workspace because the addresses to choose from depend on
 * the customer being edited, and that value only exists inside
 * `form.Subscribe` — same reason `QuoteNewForm` is split out.
 */
function InvoiceNewForm({
	organizationSlug,
	values,
	form,
	customers,
	projects,
	vatEnabled,
	isCreating,
	error,
}: InvoiceNewFormProps) {
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)

	const updateValues = (patch: Partial<NewInvoiceFormValues>) => {
		if (patch.kind !== undefined) form.setFieldValue('kind', patch.kind)
		if (patch.projectId !== undefined) {
			form.setFieldValue('projectId', patch.projectId)
		}
		if (patch.customerId !== undefined) {
			form.setFieldValue('customerId', patch.customerId)
			// The address belongs to the customer, so keeping the old one would
			// bill a new customer at another customer's address.
			form.setFieldValue('customerContextId', '')
		}
		if (patch.customerContextId !== undefined) {
			form.setFieldValue('customerContextId', patch.customerContextId)
		}
		if (patch.termsDays !== undefined) {
			form.setFieldValue('termsDays', patch.termsDays)
		}
		if (patch.notes !== undefined) form.setFieldValue('notes', patch.notes)
		if (patch.operationNature !== undefined) {
			form.setFieldValue('operationNature', patch.operationNature)
		}
		if (patch.lines !== undefined) form.setFieldValue('lines', patch.lines)
	}

	const updateLine = (index: number, patch: Partial<InvoiceLineFormValues>) => {
		const lines = [...form.state.values.lines]
		const current = lines[index]
		if (!current) return
		lines[index] = { ...current, ...patch }
		form.setFieldValue('lines', lines)
	}

	return (
		<InvoiceNewUI
			organizationSlug={organizationSlug}
			values={values}
			customers={customers}
			customerContexts={customerContexts.data?.data ?? []}
			projects={projects}
			error={error}
			isCreating={isCreating}
			isCustomerContextsLoading={customerContexts.isLoading}
			vatEnabled={vatEnabled}
			onChange={updateValues}
			onLineChange={updateLine}
			onAddLine={() => {
				const line = emptyInvoiceLine(newLineId(form.state.values.lines.length))
				form.setFieldValue('lines', [...form.state.values.lines, line])
				return line.clientId
			}}
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
					duplicateItem(form.state.values.lines, index, (line) => ({
						...line,
						clientId,
					})),
				)
				return clientId
			}}
			onRemoveLine={(index) => {
				const next = form.state.values.lines.filter(
					(_line, itemIndex) => itemIndex !== index,
				)
				form.setFieldValue(
					'lines',
					next.length > 0 ? next : [emptyInvoiceLine()],
				)
			}}
			onSubmit={() => void form.handleSubmit()}
		/>
	)
}
