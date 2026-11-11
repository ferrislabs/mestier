import { getRouteApi } from '@tanstack/react-router'
import { DocumentFeature } from '#/pages/knowledge-documents/feature/document-feature'

const route = getRouteApi(
	'/_app/o/$organizationSlug/connaissances/documents/$articleId',
)

export function DocumentRoutePage() {
	const { articleId } = route.useParams()
	return <DocumentFeature articleId={articleId} />
}
