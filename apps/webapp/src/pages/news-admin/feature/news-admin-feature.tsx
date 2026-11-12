import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useAdminNews } from '#/hooks/use-news'
import { mutationErrorMessage } from '#/lib/api-error'
import { NewsAdminList } from '#/pages/news-admin/ui/news-admin-list'
import { NewsEditorFeature } from './news-editor-feature'

type Selection =
	| { kind: 'list' }
	| { kind: 'new' }
	| { kind: 'edit'; news: Schemas.NewsResponse }

export function NewsAdminFeature() {
	const list = useAdminNews(true)
	const [selection, setSelection] = useState<Selection>({ kind: 'list' })

	if (selection.kind !== 'list') {
		return (
			<NewsEditorFeature
				key={selection.kind === 'edit' ? selection.news.id : 'new'}
				news={selection.kind === 'edit' ? selection.news : null}
				onBack={() => setSelection({ kind: 'list' })}
			/>
		)
	}

	return (
		<NewsAdminList
			news={list.data?.data ?? []}
			now={new Date()}
			loading={list.isPending}
			error={mutationErrorMessage(list.error)}
			onCreate={() => setSelection({ kind: 'new' })}
			onSelect={(news) => setSelection({ kind: 'edit', news })}
		/>
	)
}
