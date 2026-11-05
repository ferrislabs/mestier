import { EmptyState } from '#/components/empty-state'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useCreditNotes } from '#/pages/credit-notes/feature/use-credit-notes'
import { CreditNotesListUI } from '#/pages/credit-notes/ui/credit-notes-list-ui'

interface CreditNotesFeatureProps {
	query: string
	onQueryChange: (query: string) => void
}

export function CreditNotesFeature(props: CreditNotesFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<EmptyState
				title="Organisation indisponible"
				hint="La liste des avoirs nécessite une organisation active."
			/>
		)
	}

	return (
		<CreditNotesWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

function CreditNotesWorkspace({
	organizationId,
	query,
	onQueryChange,
}: CreditNotesFeatureProps & { organizationId: string }) {
	const customers = useCustomers(organizationId)
	const credit = useCreditNotes(organizationId)

	return (
		<CreditNotesListUI
			creditNotes={credit.creditNotes}
			invoices={credit.invoices}
			customers={customers.data?.data ?? []}
			query={query}
			isLoading={credit.isLoading || customers.isLoading}
			error={credit.error ?? customers.error?.message ?? null}
			onRetry={() => {
				credit.refetch()
				void customers.refetch()
			}}
			onQueryChange={onQueryChange}
		/>
	)
}
