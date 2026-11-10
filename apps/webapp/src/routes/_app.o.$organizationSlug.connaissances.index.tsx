import { createFileRoute } from '@tanstack/react-router'
import { KnowledgeFeature } from '#/pages/knowledge/feature/knowledge-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/connaissances/')({
	component: KnowledgeFeature,
})
