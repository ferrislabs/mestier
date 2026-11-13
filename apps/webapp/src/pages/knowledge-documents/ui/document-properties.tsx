import { Link2, Tag } from 'lucide-react'

export interface DocumentPropertyLink {
	key: string
	label: string
}

interface DocumentPropertiesProps {
	canManage: boolean
	tags: readonly string[]
	links: readonly DocumentPropertyLink[]
	onOpenInfo: () => void
}

const PROPERTY_CLASS =
	'-mx-1.5 inline-flex min-w-0 items-center gap-1.5 rounded-md px-1.5 py-1 text-left outline-none transition-colors hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50'

export function DocumentProperties({
	canManage,
	tags,
	links,
	onOpenInfo,
}: DocumentPropertiesProps) {
	const showTags = tags.length > 0 || canManage
	const showLinks = links.length > 0 || canManage
	if (!showTags && !showLinks) return null
	return (
		<div className="flex flex-wrap items-center gap-x-5 gap-y-1 text-muted-foreground text-sm">
			{showTags ? (
				<button type="button" className={PROPERTY_CLASS} onClick={onOpenInfo}>
					<Tag aria-hidden className="size-3.5 shrink-0" />
					<span className="sr-only">Étiquettes :</span>
					{tags.length > 0 ? (
						<span className="flex flex-wrap gap-1">
							{tags.map((tag) => (
								<span
									key={tag}
									className="rounded bg-muted px-1.5 text-foreground text-xs leading-5"
								>
									{tag}
								</span>
							))}
						</span>
					) : (
						<span>Ajouter des étiquettes</span>
					)}
				</button>
			) : null}
			{showLinks ? (
				<button type="button" className={PROPERTY_CLASS} onClick={onOpenInfo}>
					<Link2 aria-hidden className="size-3.5 shrink-0" />
					<span className="sr-only">Rattaché à :</span>
					{links.length > 0 ? (
						<span className="truncate text-foreground">
							{links.map((link) => link.label).join(', ')}
						</span>
					) : (
						<span>Rattacher à un projet, client ou devis</span>
					)}
				</button>
			) : null}
		</div>
	)
}
