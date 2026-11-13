import { ImageIcon, ImagePlus, Loader2 } from 'lucide-react'
import { cn } from '#/lib/utils'
import {
	BLOCK_TOOL_BUTTON,
	BLOCK_TOOLBAR,
} from '#/pages/knowledge-documents/ui/block-toolbar'
import { DeletedElement } from '#/pages/knowledge-documents/ui/deleted-element'

export type ImageBlockState = 'empty' | 'loading' | 'ready' | 'missing'

export const IMAGE_WIDTHS = [25, 50, 75, 100] as const

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
	if (state === 'empty' || state === 'missing') {
		if (!editable) {
			return (
				<div className="flex w-full items-center gap-3 rounded-lg bg-muted/50 px-4 py-3 text-muted-foreground text-sm">
					<ImageIcon aria-hidden className="size-4" />
					{state === 'missing' ? (
						<DeletedElement />
					) : (
						<span>Aucune image choisie.</span>
					)}
				</div>
			)
		}
		return (
			<button
				type="button"
				onClick={onPick}
				className="flex w-full items-center gap-3 rounded-lg bg-muted/50 px-4 py-3 text-left text-muted-foreground text-sm outline-none transition-colors hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50"
			>
				<ImagePlus aria-hidden className="size-4" />
				{state === 'missing' ? (
					<>
						<DeletedElement className="flex-1" />
						<span className="text-foreground">Remplacer</span>
					</>
				) : (
					<span>Ajouter une image</span>
				)}
			</button>
		)
	}

	return (
		<figure className="group/block grid w-full justify-items-center gap-1.5">
			<div style={{ width: `${width}%` }} className="relative min-h-12">
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
				{editable ? (
					<div
						contentEditable={false}
						className={cn(BLOCK_TOOLBAR, 'absolute top-2 right-2')}
					>
						{IMAGE_WIDTHS.map((preset) => (
							<button
								key={preset}
								type="button"
								aria-pressed={width === preset}
								aria-label={`Largeur ${preset} %`}
								onClick={() => onWidthChange(preset)}
								className={cn(
									BLOCK_TOOL_BUTTON,
									'tabular-nums',
									width === preset && 'bg-muted text-foreground',
								)}
							>
								{preset} %
							</button>
						))}
						<span aria-hidden className="mx-0.5 h-4 w-px bg-border" />
						<button
							type="button"
							onClick={onPick}
							className={BLOCK_TOOL_BUTTON}
						>
							Remplacer
						</button>
					</div>
				) : null}
			</div>
			{editable ? (
				<input
					type="text"
					aria-label="Légende de l’image"
					value={caption}
					maxLength={500}
					placeholder="Ajouter une légende"
					onChange={(event) => onCaptionChange(event.target.value)}
					className="w-full max-w-xl bg-transparent text-center text-muted-foreground text-sm outline-none placeholder:text-transparent focus:placeholder:text-muted-foreground/60 group-hover/block:placeholder:text-muted-foreground/60 pointer-coarse:placeholder:text-muted-foreground/60"
				/>
			) : caption !== '' ? (
				<figcaption className="text-muted-foreground text-sm">
					{caption}
				</figcaption>
			) : null}
		</figure>
	)
}
