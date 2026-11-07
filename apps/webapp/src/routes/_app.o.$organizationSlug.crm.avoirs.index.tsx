import { createFileRoute } from '@tanstack/react-router'
import { CreditNotesFeature } from '#/pages/credit-notes/feature/credit-notes-feature'

interface CreditNotesSearch {
	q?: string
}

function validateSearch(raw: Record<string, unknown>): CreditNotesSearch {
	const q =
		typeof raw.q === 'string'
			? raw.q
			: typeof raw.q === 'number'
				? String(raw.q)
				: ''
	return q.trim() !== '' ? { q } : {}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/avoirs/')({
	validateSearch,
	component: CreditNotesPage,
})

function CreditNotesPage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<CreditNotesFeature
			query={search.q ?? ''}
			onQueryChange={(q) =>
				navigate({ replace: true, search: q.trim() !== '' ? { q } : {} })
			}
		/>
	)
}
