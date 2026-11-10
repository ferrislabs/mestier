import { useStore } from '@tanstack/react-store'
import { useEffect, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useFileUrls } from '#/hooks/use-file-url'
import {
	useMarkNewsSeen,
	usePublishedNews,
	useUnseenNews,
} from '#/hooks/use-news'
import { clampIndex } from '#/pages/news/lib/carousel'
import { NewsDialog } from '#/pages/news/ui/news-dialog'
import { closeAllNews, newsStore } from '#/store/news.store'

const NO_NEWS: Schemas.NewsResponse[] = []

export function NewsGateFeature() {
	const showAll = useStore(newsStore, (state) => state.showAll)
	const unseen = useUnseenNews()
	const published = usePublishedNews(showAll)
	const markSeen = useMarkNewsSeen()
	const [unseenOpen, setUnseenOpen] = useState(false)
	const [snapshot, setSnapshot] = useState<Schemas.NewsResponse[]>(NO_NEWS)
	const [index, setIndex] = useState(0)
	const autoOpened = useRef(false)
	const seen = useRef(new Set<string>())

	const unseenList = unseen.data?.data
	useEffect(() => {
		if (autoOpened.current || !unseenList || unseenList.length === 0) return
		autoOpened.current = true
		setSnapshot(unseenList)
		setIndex(0)
		setUnseenOpen(true)
	}, [unseenList])

	const open = showAll || unseenOpen
	const list = showAll ? (published.data?.data ?? NO_NEWS) : snapshot
	const position = clampIndex(index, list.length)
	const current = list[position]
	const currentId = current?.id

	useEffect(() => {
		if (!open || !currentId || seen.current.has(currentId)) return
		seen.current.add(currentId)
		markSeen.mutate({ body: { ids: [currentId] } })
	}, [open, currentId, markSeen.mutate])

	const keys = Array.from(
		new Set(list.flatMap((news) => (news.image_key ? [news.image_key] : []))),
	)
	const previews = useFileUrls(open ? keys : [])
	const imageUrls = Object.fromEntries(
		previews.map((preview) => [preview.key, preview.url]),
	)

	if (!open) return null

	return (
		<NewsDialog
			open={open}
			news={list}
			index={position}
			imageUrls={imageUrls}
			onIndexChange={setIndex}
			onClose={() => {
				setUnseenOpen(false)
				closeAllNews()
				setIndex(0)
			}}
		/>
	)
}
