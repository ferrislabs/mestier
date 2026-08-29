import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useImportSupplierInvoice,
	useSupplierInvoices,
} from '#/hooks/use-supplier-invoices'
import { SupplierInvoiceInboxUI } from '#/pages/purchase/ui/supplier-invoice-inbox-ui'

export interface ImportOutcome {
	id: string
	fileName: string
	status: 'pending' | 'created' | 'parse_failed' | 'error'
	message?: string
}

const DEFAULT_PAGE_SIZE = 100

export function SupplierInvoiceInboxFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<SupplierInvoiceInboxWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

function SupplierInvoiceInboxWorkspace({
	organizationId,
	organizationSlug,
}: {
	organizationId: string
	organizationSlug: string
}) {
	const [page, setPage] = useState(1)
	const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE)
	const [imports, setImports] = useState<ImportOutcome[]>([])
	const supplierInvoices = useSupplierInvoices(organizationId, {
		page,
		perPage: pageSize,
	})
	const importInvoice = useImportSupplierInvoice(organizationId)

	// A batch where one file fails must not lose the others (#340's own
	// rule): each file is imported on its own, sequentially — parallel
	// requests would all race the same duplicate check and could each see
	// "not yet imported" for what is, in truth, the same document twice.
	const importFiles = async (files: File[]) => {
		for (const file of files) {
			const importId = `${file.name}-${file.size}-${file.lastModified}`
			setImports((current) => [
				{ id: importId, fileName: file.name, status: 'pending' },
				...current,
			])

			try {
				const result = await importInvoice.mutateAsync(file)
				setImports((current) =>
					current.map((entry) =>
						entry.id === importId
							? {
									...entry,
									status: result.data.outcome,
									message:
										result.data.outcome === 'parse_failed'
											? result.data.reason
											: undefined,
								}
							: entry,
					),
				)
			} catch (error) {
				setImports((current) =>
					current.map((entry) =>
						entry.id === importId
							? {
									...entry,
									status: 'error',
									message:
										error instanceof Error
											? error.message
											: 'Échec inattendu de l’import.',
								}
							: entry,
					),
				)
			}
		}
	}

	return (
		<SupplierInvoiceInboxUI
			organizationSlug={organizationSlug}
			supplierInvoices={supplierInvoices.data?.data ?? []}
			pagination={supplierInvoices.data?.pagination ?? null}
			page={page}
			pageSize={pageSize}
			isLoading={supplierInvoices.isLoading}
			error={supplierInvoices.error?.message ?? null}
			onRetry={() => void supplierInvoices.refetch()}
			onPageChange={setPage}
			onPageSizeChange={(nextPageSize) => {
				setPageSize(nextPageSize)
				setPage(1)
			}}
			imports={imports}
			onDismissImport={(id) =>
				setImports((current) => current.filter((entry) => entry.id !== id))
			}
			onImportFiles={(files) => void importFiles(files)}
			isImporting={importInvoice.isPending}
		/>
	)
}
