import type { Schemas } from '#/api/api.client'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogTitle,
} from '#/components/ui/dialog'
import { clampIndex, indexForKey } from '#/pages/news/lib/carousel'
import { NewsContent } from './news-content'

interface NewsDialogProps {
	open: boolean
	news: ReadonlyArray<Schemas.NewsResponse>
	index: number
	imageUrls: Readonly<Record<string, string | undefined>>
	onIndexChange: (index: number) => void
	onClose: () => void
}

export function NewsDialog({
	open,
	news,
	index,
	imageUrls,
	onIndexChange,
	onClose,
}: NewsDialogProps) {
	const current = news[clampIndex(index, news.length)]
	if (!current) return null
	const position = clampIndex(index, news.length)
	const imageUrl = current.image_key ? imageUrls[current.image_key] : undefined

	return (
		<Dialog
			open={open}
			onOpenChange={(next) => {
				if (!next) onClose()
			}}
		>
			<DialogContent
				showCloseButton={false}
				className="gap-0 overflow-hidden p-0 sm:max-w-xl"
				onKeyDown={(event) => {
					const target = indexForKey(event.key, position, news.length)
					if (target !== null && target !== position) onIndexChange(target)
				}}
			>
				<DialogTitle className="sr-only">Nouveautés</DialogTitle>
				<DialogDescription className="sr-only">
					Les dernières nouveautés de Mestier.
				</DialogDescription>
				<div className="max-h-[85vh] overflow-y-auto">
					<NewsContent
						news={current}
						imageUrl={imageUrl}
						index={position}
						count={news.length}
						onIndexChange={onIndexChange}
						onClose={onClose}
					/>
				</div>
			</DialogContent>
		</Dialog>
	)
}
