import { createFileRoute } from '@tanstack/react-router'
import { CreditNoteDetailFeature } from '#/pages/credit-notes/feature/credit-note-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/credit-notes/$creditNoteId',
)({
	component: CreditNotePage,
})

function CreditNotePage() {
	const { creditNoteId } = Route.useParams()
	return (
		<CreditNoteDetailFeature key={creditNoteId} creditNoteId={creditNoteId} />
	)
}
