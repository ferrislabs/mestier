export function DaySeparator({ label }: { label: string }) {
	return (
		<h2 className="mt-4 mb-1 flex items-center gap-3 px-4 font-semibold text-muted-foreground text-xs">
			<span aria-hidden className="h-px flex-1 bg-border" />
			<span className="rounded-full border border-border bg-card px-3 py-0.5">
				{label}
			</span>
			<span aria-hidden className="h-px flex-1 bg-border" />
		</h2>
	)
}
