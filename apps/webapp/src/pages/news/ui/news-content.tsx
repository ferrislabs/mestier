import { ChevronLeft, ChevronRight, Sparkles, X } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { newsIconComponent } from '#/pages/news/lib/news-icons'

export interface NewsContentData {
	title: string
	items: ReadonlyArray<Schemas.NewsItemResponse>
	cta?: Schemas.NewsCtaResponse | null
}

interface NewsContentProps {
	news: NewsContentData
	imageUrl?: string
	index: number
	count: number
	onIndexChange: (index: number) => void
	onClose?: () => void
	className?: string
}

export function NewsContent({
	news,
	imageUrl,
	index,
	count,
	onIndexChange,
	onClose,
	className,
}: NewsContentProps) {
	const carousel = count > 1

	return (
		<div
			className={cn(
				'flex flex-col overflow-hidden rounded-lg bg-background',
				className,
			)}
		>
			<div className="relative aspect-[2/1] w-full shrink-0 overflow-hidden">
				{imageUrl ? (
					<img
						src={imageUrl}
						alt=""
						className="size-full object-cover"
						draggable={false}
					/>
				) : (
					<div
						data-testid="news-banner-fallback"
						className="flex size-full items-center justify-center bg-gradient-to-br from-primary to-primary/50 text-primary-foreground"
					>
						<Sparkles className="size-12 opacity-80" aria-hidden />
					</div>
				)}
				{onClose ? (
					<Button
						type="button"
						variant="secondary"
						size="icon-sm"
						aria-label="Fermer"
						className="absolute top-3 right-3 rounded-full shadow-sm"
						onClick={onClose}
					>
						<X />
					</Button>
				) : null}
			</div>
			<div className="flex flex-col gap-5 px-6 pt-6 pb-6">
				<h2 className="break-words font-semibold text-xl tracking-tight">
					{news.title}
				</h2>
				<ul className="flex flex-col gap-4">
					{news.items.map((item, position) => {
						const Icon = newsIconComponent(item.icon)
						return (
							<li
								// biome-ignore lint/suspicious/noArrayIndexKey: points have no id
								key={`${position}-${item.title}`}
								className="flex items-start gap-3"
							>
								<Icon
									className="mt-0.5 size-4 shrink-0 text-foreground"
									aria-hidden
								/>
								<div className="min-w-0">
									<p className="break-words font-medium text-sm">
										{item.title}
									</p>
									<p className="mt-0.5 break-words text-muted-foreground text-sm">
										{item.text}
									</p>
								</div>
							</li>
						)
					})}
				</ul>
			</div>
			<div className="flex items-center justify-between gap-3 border-t px-6 py-3">
				{carousel ? (
					<div className="flex items-center gap-2">
						<Button
							type="button"
							variant="ghost"
							size="icon-sm"
							aria-label="Nouveauté précédente"
							disabled={index <= 0}
							onClick={() => onIndexChange(index - 1)}
						>
							<ChevronLeft />
						</Button>
						<div className="flex items-center gap-1.5">
							{Array.from({ length: count }, (_, position) => (
								<button
									// biome-ignore lint/suspicious/noArrayIndexKey: dots are positional
									key={position}
									type="button"
									aria-label={`Nouveauté ${position + 1} sur ${count}`}
									aria-current={position === index ? 'true' : undefined}
									className={cn(
										'size-2 rounded-full transition-colors focus-visible:ring-2 focus-visible:ring-ring',
										position === index
											? 'bg-foreground'
											: 'bg-muted-foreground/30 hover:bg-muted-foreground/60',
									)}
									onClick={() => onIndexChange(position)}
								/>
							))}
						</div>
						<Button
							type="button"
							variant="ghost"
							size="icon-sm"
							aria-label="Nouveauté suivante"
							disabled={index >= count - 1}
							onClick={() => onIndexChange(index + 1)}
						>
							<ChevronRight />
						</Button>
					</div>
				) : (
					<span />
				)}
				{news.cta ? (
					<Button asChild>
						<a href={news.cta.url} target="_blank" rel="noopener noreferrer">
							{news.cta.label}
						</a>
					</Button>
				) : onClose ? (
					<Button type="button" onClick={onClose}>
						Compris
					</Button>
				) : null}
			</div>
		</div>
	)
}
