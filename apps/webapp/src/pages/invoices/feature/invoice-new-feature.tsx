import { useForm } from '@tanstack/react-form'
import { useNavigate } from '@tanstack/react-router'
import { AlertCircle } from 'lucide-react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	useCustomerContexts,
	useCustomers,
} from '#/hooks/use-customers'
import { useCreateInvoice } from '#/hooks/use-invoices'
import { type Project, useProjects } from '#/hooks/use-projects'
import { buildOrgPath } from '#/modules/org-path'
import {
	emptyInvoiceLine,
	eurosToCents,
	type InvoiceFormValues,
	type InvoiceLineFormValues,
} from '#/pages/invoices/types'
import { InvoiceNewUI } from '#/pages/invoices/ui/invoice-new-ui'

export function InvoiceNewFeature() {
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
						La création de facture nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<InvoiceNewWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			vatEnabled={activeOrganization.vat_status?.type === 'subject'}
		/>
	)
}

function InvoiceNewWorkspace({
	organizationId,
	organizationSlug,
	vatEnabled,
}: {
	organizationId: string
	organizationSlug: string
	vatEnabled: boolean
}) {
	const navigate = useNavigate()
	const customers = useCustomers(organizationId)
	const projects = useProjects(organizationId, { includeArchived: false })
	const createInvoice = useCreateInvoice(organizationId)

	const form = useForm({
		defaultValues: {
			kind: 'STANDARD',
			projectId: '',
			customerId: '',
			customerContextId: '',
			dueAt: '',
			notes: '',
			operationNature: '',
			lines: [emptyInvoiceLine()],
		} satisfies InvoiceFormValues,
		onSubmit: async ({ value }) => {
			const invoice = await createInvoice.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					kind: value.kind,
					project_id: value.projectId || null,
					customer_id: value.customerId,
					customer_context_id: value.customerContextId,
					due_at: value.dueAt ? new Date(value.dueAt).toISOString() : null,
					notes: value.notes.trim() || null,
					operation_nature: value.operationNature || null,
					lines: value.lines.map((line) => ({
						label: line.label.trim(),
						quantity: line.quantity.replace(',', '.').trim(),
						unit_price_cents: eurosToCents(line.unitPrice),
						vat_rate_basis_points:
							line.vatRateBp === '' ? null : Number(line.vatRateBp),
					})),
				},
			})

			// Straight to the invoice just created, same reasoning as
			// `QuoteNewFeature`: the next thing an artisan does — issue it,
			// record a payment, export the PDF — lives on that page.
			await navigate({
				to: buildOrgPath(organizationSlug, '/crm/invoices/$invoiceId'),
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
	state: { values: InvoiceFormValues }
	setFieldValue: (
		field: keyof InvoiceFormValues,
		value: InvoiceFormValues[keyof InvoiceFormValues],
	) => void
	handleSubmit: () => void | Promise<void>
}

interface InvoiceNewFormProps {
	organizationSlug: string
	values: InvoiceFormValues
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

	const updateValues = (patch: Partial<InvoiceFormValues>) => {
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
		if (patch.dueAt !== undefined) form.setFieldValue('dueAt', patch.dueAt)
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
			onAddLine={() =>
				form.setFieldValue('lines', [
					...form.state.values.lines,
					emptyInvoiceLine(
						`line-${Date.now()}-${form.state.values.lines.length}`,
					),
				])
			}
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
