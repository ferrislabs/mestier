import { Loader2 } from 'lucide-react'
import { PageShell, SectionCard } from '#/components/ui/surface'

export function RhLoading({ label }: { label: string }) {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				{label}
			</SectionCard>
		</PageShell>
	)
}

export function RhLoadError({ message }: { message: string }) {
	return (
		<PageShell>
			<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
				{message}
			</div>
		</PageShell>
	)
}
