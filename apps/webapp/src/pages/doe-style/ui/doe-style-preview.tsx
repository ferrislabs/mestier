import { Loader2 } from 'lucide-react'
import { cn } from '#/lib/utils'

export const PREVIEW_PAGE_LABELS = [
	'Couverture',
	'Intervenants',
	'Fiche de fourniture',
]

interface DoeStylePreviewProps {
	pages: string[]
	activePage: number
	loading: boolean
	error: string | null
	invalid: boolean
	onSelectPage: (index: number) => void
}

export function svgDataUrl(svg: string): string {
	return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`
}

export function DoeStylePreview({
	pages,
	activePage,
	loading,
	error,
	invalid,
	onSelectPage,
}: DoeStylePreviewProps) {
	const current = Math.min(activePage, Math.max(pages.length - 1, 0))
	const page = pages[current]

	return (
		<div className="flex flex-col gap-3">
			{error ? (
				<p role="alert" className="text-sm text-destructive">
					{error}
				</p>
			) : null}
			{invalid ? (
				<p className="text-xs text-muted-foreground">
					Corrigez les champs en erreur pour actualiser l'aperçu.
				</p>
			) : null}
			<div className="relative">
				<div
					className={cn(
						'aspect-[210/297] w-full overflow-hidden rounded-sm bg-white shadow-lg ring-1 ring-black/5 transition-opacity',
						loading ? 'opacity-80' : 'opacity-100',
					)}
				>
					{page ? (
						<img
							src={svgDataUrl(page)}
							alt={`Aperçu : ${PREVIEW_PAGE_LABELS[current] ?? `page ${current + 1}`}`}
							className="size-full object-contain"
						/>
					) : (
						<div className="flex size-full items-center justify-center text-sm text-muted-foreground">
							{loading ? "Génération de l'aperçu…" : 'Aperçu indisponible'}
						</div>
					)}
				</div>
				{loading ? (
					<output
						aria-label="Aperçu en cours de mise à jour"
						className="absolute top-2 right-2 rounded-full bg-background/90 p-1 shadow"
					>
						<Loader2 className="size-3.5 animate-spin text-muted-foreground" />
					</output>
				) : null}
			</div>
			{pages.length > 1 ? (
				<div className="grid grid-cols-3 gap-2">
					{pages.map((svg, index) => (
						<button
							// biome-ignore lint/suspicious/noArrayIndexKey: pages are positional
							key={index}
							type="button"
							aria-label={`Afficher : ${PREVIEW_PAGE_LABELS[index] ?? `page ${index + 1}`}`}
							aria-pressed={index === current}
							onClick={() => onSelectPage(index)}
							className={cn(
								'flex flex-col gap-1 rounded-md border p-1 text-left transition',
								index === current
									? 'border-primary ring-2 ring-primary/30'
									: 'border-transparent hover:border-border',
							)}
						>
							<span className="block aspect-[210/297] overflow-hidden rounded-sm bg-white shadow ring-1 ring-black/5">
								<img
									src={svgDataUrl(svg)}
									alt=""
									className="size-full object-contain"
								/>
							</span>
							<span className="truncate text-xs text-muted-foreground">
								{PREVIEW_PAGE_LABELS[index] ?? `Page ${index + 1}`}
							</span>
						</button>
					))}
				</div>
			) : null}
		</div>
	)
}
