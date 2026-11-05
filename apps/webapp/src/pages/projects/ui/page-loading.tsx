import { Loader2 } from 'lucide-react'
import { Page } from '#/pages/projects/ui/page'

export function PageLoading({ label }: { label: string }) {
	return (
		<Page>
			<output className="flex min-h-48 items-center justify-center gap-3 rounded-lg border bg-card p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" aria-hidden />
				{label}
			</output>
		</Page>
	)
}
