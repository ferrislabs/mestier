import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useFileUrls } from '#/hooks/use-file-url'
import { useProjects } from '#/hooks/use-projects'
import {
	useConfirmSupplierInvoice,
	useRejectSupplierInvoice,
	useReplaceLineAllocations,
	useSupplierInvoice,
	useUpdateSupplierInvoiceNotes,
} from '#/hooks/use-supplier-invoices'
import { SupplierInvoiceDetailUI } from '#/pages/purchase/ui/supplier-invoice-detail-ui'

interface SupplierInvoiceDetailFeatureProps {
	supplierInvoiceId: string
}

export function SupplierInvoiceDetailFeature({
	supplierInvoiceId,
}: SupplierInvoiceDetailFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const invoice = useSupplierInvoice(supplierInvoiceId)
	const projects = useProjects(activeOrganization.id, {
		includeArchived: false,
	})
	const sourceFileKey = invoice.data?.data.source_file_key
	const fileUrls = useFileUrls(sourceFileKey ? [sourceFileKey] : [])

	const updateNotes = useUpdateSupplierInvoiceNotes()
	const confirm = useConfirmSupplierInvoice()
	const reject = useRejectSupplierInvoice()
	const replaceAllocations = useReplaceLineAllocations()

	if (invoice.isLoading) {
		return <SupplierInvoiceDetailUI.Loading />
	}

	if (invoice.isError || !invoice.data?.data) {
		return (
			<SupplierInvoiceDetailUI.ErrorState
				organizationSlug={activeOrganization.slug}
				message={
					invoice.error?.message ??
					'Aucune facture ne correspond à cet identifiant.'
				}
			/>
		)
	}

	return (
		<SupplierInvoiceDetailUI
			organizationSlug={activeOrganization.slug}
			invoice={invoice.data.data}
			projects={projects.data?.data ?? []}
			fileUrl={fileUrls[0]?.url}
			onSaveNotes={(notes) =>
				void updateNotes.mutateAsync({
					path: { supplier_invoice_id: supplierInvoiceId },
					body: { notes },
				})
			}
			isSavingNotes={updateNotes.isPending}
			onConfirm={(notes) =>
				void confirm.mutateAsync({
					path: { supplier_invoice_id: supplierInvoiceId },
					body: { notes },
				})
			}
			isConfirming={confirm.isPending}
			onReject={(notes) =>
				void reject.mutateAsync({
					path: { supplier_invoice_id: supplierInvoiceId },
					body: { notes },
				})
			}
			isRejecting={reject.isPending}
			onSaveLineAllocations={(lineId, shares) =>
				void replaceAllocations.mutateAsync({
					path: { supplier_invoice_line_id: lineId },
					body: {
						allocations: shares.map((share) => ({
							project_id: share.projectId,
							amount_cents: share.amountCents,
						})),
					},
				})
			}
			isSavingAllocations={replaceAllocations.isPending}
		/>
	)
}
