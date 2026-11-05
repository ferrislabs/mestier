export function PhotoPreview({
	name,
	sizeLabel,
	url,
}: {
	name: string
	sizeLabel: string
	url: string
}) {
	return (
		<figure className="mt-2 w-full max-w-sm">
			<a href={url} target="_blank" rel="noreferrer">
				<img
					src={url}
					alt={name}
					className="aspect-[3/2] w-full rounded-lg border border-border object-cover"
				/>
			</a>
			<figcaption className="mt-1 text-muted-foreground text-xs">
				{name} · {sizeLabel}
			</figcaption>
		</figure>
	)
}
