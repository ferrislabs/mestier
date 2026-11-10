import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type KnowledgeResourceType,
	useKnowledgeArticles,
	useLinkKnowledgeArticle,
	useResourceKnowledge,
	useUnlinkKnowledgeArticle,
} from '#/hooks/use-knowledge'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { KnowledgeDetailFeature } from '#/pages/knowledge/feature/knowledge-detail-feature'
import {
	AttachKnowledgeDialog,
	LinkedKnowledge,
} from '#/pages/knowledge/ui/linked-knowledge'

const NO_FILTERS = { q: '', tags: [] }

interface LinkedKnowledgeFeatureProps {
	resourceType: KnowledgeResourceType
	resourceId: string
}

/**
 * The documents and notes attached to a project, a customer or a quote. Shown
 * to whoever can read the base, editable by whoever can manage it.
 */
export function LinkedKnowledgeFeature(props: LinkedKnowledgeFeatureProps) {
	const canView = useHasPermission('VIEW_KNOWLEDGE')
	const { activeOrganization } = useActiveOrganization()
	if (!canView) return null
	return (
		<LinkedKnowledgeSection
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

function LinkedKnowledgeSection({
	organizationId,
	resourceType,
	resourceId,
}: LinkedKnowledgeFeatureProps & { organizationId: string }) {
	const canManage = useHasPermission('MANAGE_KNOWLEDGE')
	const [openId, setOpenId] = useState<string | null>(null)
	const [attaching, setAttaching] = useState(false)
	const [query, setQuery] = useState('')

	const linked = useResourceKnowledge(organizationId, resourceType, resourceId)
	const link = useLinkKnowledgeArticle(organizationId)
	const unlink = useUnlinkKnowledgeArticle(organizationId)
	const candidates = useKnowledgeArticles(
		organizationId,
		useMemo(() => ({ ...NO_FILTERS, q: query.trim() }), [query]),
		1,
	)

	const linkedIds = new Set((linked.data ?? []).map((article) => article.id))
	const attachable = (candidates.data?.articles ?? []).filter(
		(article) => !linkedIds.has(article.id),
	)

	return (
		<>
			<LinkedKnowledge
				articles={linked.data ?? []}
				isLoading={linked.isLoading}
				error={mutationErrorMessage(linked.error ?? unlink.error ?? null)}
				canManage={canManage}
				onOpen={(article) => setOpenId(article.id)}
				onUnlink={(article) =>
					unlink.mutate({
						articleId: article.id,
						resourceType,
						resourceId,
					})
				}
				onAttach={() => setAttaching(true)}
			/>
			{attaching ? (
				<AttachKnowledgeDialog
					open
					query={query}
					candidates={attachable}
					isLoading={candidates.isLoading}
					isAttaching={link.isPending}
					error={mutationErrorMessage(link.error)}
					onQueryChange={setQuery}
					onPick={(article) =>
						link.mutate(
							{ articleId: article.id, resourceType, resourceId },
							{
								onSuccess: () => {
									setAttaching(false)
									setQuery('')
								},
							},
						)
					}
					onOpenChange={(open) => {
						if (!open) {
							setAttaching(false)
							setQuery('')
						}
					}}
				/>
			) : null}
			{openId ? (
				<KnowledgeDetailFeature
					key={openId}
					organizationId={organizationId}
					articleId={openId}
					canManage={canManage}
					onClose={() => setOpenId(null)}
				/>
			) : null}
		</>
	)
}
