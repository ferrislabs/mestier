import { createFileRoute } from '@tanstack/react-router'

import { NewsAdminFeature } from '#/pages/news-admin/feature/news-admin-feature'

export const Route = createFileRoute('/_app/admin/news')({
	component: NewsAdminFeature,
})
