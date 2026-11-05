import { Download, FileText } from 'lucide-react'

export function AttachmentCard({
	name,
	typeLabel,
	sizeLabel,
	url,
}: {
	name: string
	typeLabel: string
	sizeLabel: string
	url: string | undefined
}) {
	return (
		<div className="mt-2 flex w-full max-w-sm items-center gap-3 rounded-lg border border-border bg-card p-2.5">
			<span className="flex size-10 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
				<FileText className="size-5" aria-hidden />
			</span>
			<div className="min-w-0 flex-1">
				<p className="truncate font-medium text-sm">{name}</p>
				<p className="text-muted-foreground text-xs">
					{typeLabel} · {sizeLabel}
				</p>
			</div>
			{url ? (
				<a
					href={url}
					target="_blank"
					rel="noreferrer"
					aria-label={`Télécharger ${name}`}
					className="inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground outline-none transition-colors hover:bg-accent hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
				>
					<Download className="size-4" aria-hidden />
				</a>
			) : null}
		</div>
	)
}
