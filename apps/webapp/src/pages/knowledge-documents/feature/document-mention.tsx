import { createReactInlineContentSpec } from '@blocknote/react'
import { useCustomer } from '#/hooks/use-customers'
import { useProject } from '#/hooks/use-projects'
import { useQuote } from '#/hooks/use-quotes'
import { isNotFoundError } from '#/lib/api-error'
import { useDocumentEnvironment } from '#/pages/knowledge-documents/feature/document-environment'
import type { MentionKind } from '#/pages/knowledge-documents/lib/document-api-types'
import { MentionChip } from '#/pages/knowledge-documents/ui/mention-chip'

interface Resolved {
	label: string
	missing: boolean
}

function useResolved(
	kind: MentionKind,
	id: string,
	storedLabel: string,
): Resolved {
	const { organizationId } = useDocumentEnvironment()
	const customer = useCustomer(id, kind === 'customer')
	const project = useProject(organizationId, kind === 'project' ? id : '')
	const quote = useQuote(id, kind === 'quote')
	const query =
		kind === 'customer' ? customer : kind === 'project' ? project : quote
	if (query.isError) {
		return { label: storedLabel, missing: isNotFoundError(query.error) }
	}
	const data = query.data?.data as
		| { name?: string; reference?: string | null }
		| undefined
	const current = kind === 'quote' ? data?.reference : data?.name
	return { label: current || storedLabel, missing: false }
}

export const mentionInlineSpec = createReactInlineContentSpec(
	{
		type: 'mention',
		propSchema: {
			resourceType: { default: 'customer' },
			resourceId: { default: '' },
			label: { default: '' },
		},
		content: 'none',
	},
	{
		render: ({ inlineContent }) => {
			const environment = useDocumentEnvironment()
			const kind = inlineContent.props.resourceType as MentionKind
			const id = inlineContent.props.resourceId
			const resolved = useResolved(kind, id, inlineContent.props.label)
			return (
				<MentionChip
					label={resolved.label}
					missing={resolved.missing}
					href={environment.resourceHref(kind, id)}
					onNavigate={() => environment.openResource(kind, id)}
				/>
			)
		},
	},
)
