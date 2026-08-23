import { Link, useNavigate } from '@tanstack/react-router'
import { AlertCircle, ArrowLeft, Loader2 } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomer, useCustomerContexts } from '#/hooks/use-customers'
import {
	useCreateInvoice,
	useInvoicesByProject,
	useIssueDeposit,
	useIssueFinalInvoice,
	useIssueInvoice,
	useProjectBillingSummary,
} from '#/hooks/use-invoices'
import { type Project, useProject } from '#/hooks/use-projects'
import { useProfitability } from '#/hooks/use-reporting'
import { buildOrgPath } from '#/modules/org-path'
import { eurosToCents } from '#/pages/invoices/types'
import {
	canInvoiceAgainstQuote,
	depositPreviewCents,
	type InvoicerMode,
	percentToBasisPoints,
} from '#/pages/projects/types'
import { ProjectDetailUI } from '#/pages/projects/ui/project-detail-ui'
import { currentMonthPeriod } from '#/pages/reporting/types'

interface ProjectDetailFeatureProps {
	projectId: string
}

export function ProjectDetailFeature({ projectId }: ProjectDetailFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const project = useProject(activeOrganization.id, projectId)

	if (project.isLoading) return <DetailLoading />

	if (project.isError) {
		return (
			<DetailError
				organizationSlug={activeOrganization.slug}
				message={project.error.message}
				onRetry={() => void project.refetch()}
			/>
		)
	}

	if (!project.data?.data) {
		return (
			<DetailError
				organizationSlug={activeOrganization.slug}
				message="Aucun projet ne correspond à cet identifiant."
			/>
		)
	}

	return (
		<ProjectDetailWorkspace
			project={project.data.data}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

function ProjectDetailWorkspace({
	project,
	organizationId,
	organizationSlug,
}: {
	project: Project
	organizationId: string
	organizationSlug: string
}) {
	const navigate = useNavigate()
	const hasCustomer = project.customer_id !== null
	const hasPinnedAddress = project.customer_context_id !== null

	// Never recomputed from a list of invoices — see `useProjectBillingSummary`.
	const billingSummary = useProjectBillingSummary(project.id)
	const invoices = useInvoicesByProject(project.id)
	const customer = useCustomer(project.customer_id ?? '', hasCustomer)

	// The report is period-scoped; a project detail page can pick its own
	// range the same way the reporting page does, rather than either fetching
	// the organization's whole history or hard-coding one window.
	const [period, setPeriod] = useState(() => currentMonthPeriod(new Date()))
	const profitability = useProfitability(organizationId, period)
	const costRow =
		profitability.data?.data.projects.find(
			(row) => row.project_id === project.id,
		) ?? null

	const freeAmountCustomerContexts = useCustomerContexts(
		project.customer_id ?? '',
		hasCustomer,
	)

	const [mode, setMode] = useState<InvoicerMode>('REMAINING')
	const [percentageInput, setPercentageInput] = useState('')
	const [freeAmountInput, setFreeAmountInput] = useState('')
	const [freeLabel, setFreeLabel] = useState('')
	const [freeCustomerContextId, setFreeCustomerContextId] = useState(
		project.customer_context_id ?? '',
	)
	const [dueAt, setDueAt] = useState('')
	const [notes, setNotes] = useState('')

	const issueDeposit = useIssueDeposit()
	const issueFinal = useIssueFinalInvoice()
	const createInvoice = useCreateInvoice(organizationId)
	const issueInvoice = useIssueInvoice()

	const quotedCents = billingSummary.data?.data.quoted_cents ?? null
	const hasQuote = canInvoiceAgainstQuote(quotedCents)
	const percentageBp = percentToBasisPoints(percentageInput)

	const goToInvoice = (invoiceId: string) =>
		navigate({
			to: buildOrgPath(organizationSlug, '/crm/invoices/$invoiceId'),
			params: { invoiceId },
		})

	const dueAtIso = () => (dueAt ? new Date(dueAt).toISOString() : null)

	const confirmRemaining = async () => {
		if (!hasQuote || !hasPinnedAddress) return
		const invoice = await issueFinal.mutateAsync({
			path: { project_id: project.id },
			body: { due_at: dueAtIso(), notes: notes.trim() || null },
		})
		await goToInvoice(invoice.data.id)
	}

	const confirmPercentage = async () => {
		if (!hasQuote || !hasPinnedAddress || percentageBp === null) return
		const invoice = await issueDeposit.mutateAsync({
			path: { project_id: project.id },
			body: {
				percentage_bp: percentageBp,
				due_at: dueAtIso(),
				notes: notes.trim() || null,
			},
		})
		await goToInvoice(invoice.data.id)
	}

	const confirmFreeAmount = async () => {
		if (
			!hasCustomer ||
			!project.customer_id ||
			!freeCustomerContextId ||
			!freeLabel.trim() ||
			!freeAmountInput.trim()
		) {
			return
		}

		// Two round trips, not one: there is no backend command for "an
		// arbitrary amount tied to a project", so this composes the same two
		// calls a manually created invoice already goes through from the
		// invoices module (`InvoiceNewFeature` then `useIssueInvoice`).
		const draft = await createInvoice.mutateAsync({
			path: { organization_id: organizationId },
			body: {
				kind: 'STANDARD',
				project_id: project.id,
				customer_id: project.customer_id,
				customer_context_id: freeCustomerContextId,
				due_at: dueAtIso(),
				notes: notes.trim() || null,
				operation_nature: null,
				lines: [
					{
						label: freeLabel.trim(),
						quantity: '1',
						unit_price_cents: eurosToCents(freeAmountInput),
						vat_rate_basis_points: null,
					},
				],
			},
		})
		const issued = await issueInvoice.mutateAsync({
			path: { invoice_id: draft.data.id },
			body: {},
		})
		await goToInvoice(issued.data.id)
	}

	const confirmHandlers: Record<InvoicerMode, () => void> = {
		REMAINING: () => void confirmRemaining(),
		PERCENTAGE: () => void confirmPercentage(),
		FREE_AMOUNT: () => void confirmFreeAmount(),
	}

	const isIssuing =
		issueDeposit.isPending ||
		issueFinal.isPending ||
		createInvoice.isPending ||
		issueInvoice.isPending

	const error =
		billingSummary.error?.message ??
		invoices.error?.message ??
		issueDeposit.error?.message ??
		issueFinal.error?.message ??
		createInvoice.error?.message ??
		issueInvoice.error?.message ??
		null

	return (
		<ProjectDetailUI
			organizationSlug={organizationSlug}
			project={project}
			customerName={customer.data?.data.name ?? null}
			billingSummary={billingSummary.data?.data ?? null}
			isBillingSummaryLoading={billingSummary.isLoading}
			period={period}
			onPeriodChange={setPeriod}
			costRow={costRow}
			isCostLoading={profitability.isLoading}
			invoices={invoices.data?.data ?? []}
			isInvoicesLoading={invoices.isLoading}
			hasCustomer={hasCustomer}
			hasPinnedAddress={hasPinnedAddress}
			hasQuote={hasQuote}
			mode={mode}
			onModeChange={setMode}
			percentageInput={percentageInput}
			onPercentageInputChange={setPercentageInput}
			percentagePreviewCents={
				hasQuote && quotedCents !== null && percentageBp !== null
					? depositPreviewCents(quotedCents, percentageBp)
					: null
			}
			freeAmountInput={freeAmountInput}
			onFreeAmountInputChange={setFreeAmountInput}
			freeLabel={freeLabel}
			onFreeLabelChange={setFreeLabel}
			freeCustomerContexts={freeAmountCustomerContexts.data?.data ?? []}
			isFreeCustomerContextsLoading={freeAmountCustomerContexts.isLoading}
			freeCustomerContextId={freeCustomerContextId}
			onFreeCustomerContextIdChange={setFreeCustomerContextId}
			dueAt={dueAt}
			onDueAtChange={setDueAt}
			notes={notes}
			onNotesChange={setNotes}
			isIssuing={isIssuing}
			error={error}
			onConfirm={() => confirmHandlers[mode]()}
		/>
	)
}

function DetailLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement du projet…
			</SectionCard>
		</PageShell>
	)
}

function DetailError({
	organizationSlug,
	message,
	onRetry,
}: {
	organizationSlug: string
	message: string
	onRetry?: () => void
}) {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
				<AlertCircle className="size-6 text-destructive" />
				<div>
					<p className="font-semibold">Impossible de charger le projet</p>
					<p className="mt-1 text-sm text-muted-foreground">{message}</p>
				</div>
				<div className="flex gap-2">
					<Button asChild variant="outline">
						<Link to={buildOrgPath(organizationSlug, '/planning/projects')}>
							<ArrowLeft />
							Retour aux projets
						</Link>
					</Button>
					{onRetry ? (
						<Button type="button" onClick={onRetry}>
							Réessayer
						</Button>
					) : null}
				</div>
			</SectionCard>
		</PageShell>
	)
}
