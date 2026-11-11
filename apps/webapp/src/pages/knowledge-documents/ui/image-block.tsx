import { ImageIcon, Loader2 } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import { DeletedElement } from '#/pages/knowledge-documents/ui/deleted-element'

export type ImageBlockState = 'empty' | 'loading' | 'ready' | 'missing'

interface ImageBlockProps {
	state: ImageBlockState
	url: string | null
	caption: string
	width: number
	editable: boolean
	onPick: () => void
	onCaptionChange: (caption: string) => void
	onWidthChange: (width: number) => void
}

export function ImageBlock({
	state,
	url,
	caption,
	width,
	editable,
	onPick,
	onCaptionChange,
	onWidthChange,
}: ImageBlockProps) {
	const baseId = useId()

	if (state === 'empty') {
		return (
			<div className="flex w-full items-center gap-3 rounded-lg border border-dashed px-4 py-6 text-muted-foreground text-sm">
				<ImageIcon aria-hidden className="size-5" />
				<span className="flex-1">Aucune image choisie.</span>
				{editable ? (
					<Button type="button" size="sm" variant="outline" onClick={onPick}>
						Choisir une image
					</Button>
				) : null}
			</div>
		)
	}

	if (state === 'missing') {
		return (
			<div className="flex w-full items-center gap-3 rounded-lg border border-dashed px-4 py-6 text-sm">
				<ImageIcon aria-hidden className="size-5 text-muted-foreground" />
				<DeletedElement className="flex-1" />
				{editable ? (
					<Button type="button" size="sm" variant="outline" onClick={onPick}>
						Remplacer
					</Button>
				) : null}
			</div>
		)
	}

	return (
		<figure className="grid w-full justify-items-center gap-2">
			<div style={{ width: `${width}%` }} className="min-h-12">
				{state === 'loading' || !url ? (
					<div className="flex h-40 items-center justify-center rounded-lg bg-muted text-muted-foreground">
						<Loader2 aria-hidden className="size-5 animate-spin" />
						<span className="sr-only">Chargement de l’image</span>
					</div>
				) : (
					<img
						src={url}
						alt={caption}
						className="h-auto w-full rounded-lg object-contain"
					/>
				)}
			</div>
			{editable ? (
				<div className="flex w-full max-w-xl flex-wrap items-center gap-3 text-sm">
					<input
						type="text"
						aria-label="Légende de l’image"
						value={caption}
						maxLength={500}
						placeholder="Légende"
						onChange={(event) => onCaptionChange(event.target.value)}
						className="min-w-40 flex-1 rounded-md border bg-transparent px-2 py-1 outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
					/>
					<label
						htmlFor={`${baseId}-width`}
						className="flex items-center gap-2 text-muted-foreground"
					>
						Largeur
						<input
							id={`${baseId}-width`}
							type="range"
							min={10}
							max={100}
							step={5}
							value={width}
							onChange={(event) => onWidthChange(Number(event.target.value))}
						/>
						<span className="w-10 tabular-nums">{width} %</span>
					</label>
					<Button type="button" size="sm" variant="ghost" onClick={onPick}>
						Changer
					</Button>
				</div>
			) : caption !== '' ? (
				<figcaption className="text-muted-foreground text-sm">
					{caption}
				</figcaption>
			) : null}
		</figure>
	)
}
