import { Download, Loader2 } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'

export interface DafPdfPreviewProps {
	/** The "back" link, rendered by the caller so it owns the route. */
	back: ReactNode
	subtitle: string
	pdfUrl: string | null
	isLoading: boolean
	error: string | null
	isDownloading: boolean
	onDownload: () => void
}

export function DafPdfPreview({
	back,
	subtitle,
	pdfUrl,
	isLoading,
	error,
	isDownloading,
	onDownload,
}: DafPdfPreviewProps) {
	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<div className="sticky top-0 z-10 flex flex-wrap items-center gap-x-4 gap-y-2 border-b bg-background/95 px-6 py-3 backdrop-blur">
				{back}
				<div className="min-w-0 flex-1">
					<h1 className="truncate font-semibold text-lg leading-tight">
						Aperçu du PDF
					</h1>
					<p className="truncate text-[13px] text-muted-foreground">
						{subtitle}
					</p>
				</div>
				<Button
					type="button"
					disabled={isDownloading || !pdfUrl}
					onClick={onDownload}
				>
					{isDownloading ? (
						<Loader2 className="animate-spin" aria-hidden />
					) : (
						<Download aria-hidden />
					)}
					Télécharger le PDF
				</Button>
			</div>
			<div className="flex min-h-0 flex-1 flex-col bg-muted/60">
				{isLoading ? (
					<div className="flex min-h-48 flex-1 items-center justify-center gap-3 text-muted-foreground text-sm">
						<Loader2 className="size-5 animate-spin" aria-hidden />
						Génération du PDF…
					</div>
				) : error || !pdfUrl ? (
					<p
						role="alert"
						className="m-6 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
					>
						{error ?? 'Le PDF est indisponible.'}
					</p>
				) : (
					<iframe
						title={`Aperçu du PDF : ${subtitle}`}
						src={pdfUrl}
						className="min-h-[70vh] w-full flex-1 border-0"
					/>
				)}
			</div>
		</div>
	)
}
