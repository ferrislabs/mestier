import { TriangleAlert } from 'lucide-react'
import type { ReactNode } from 'react'

export function ErrorBox({
	message,
	detail,
	hint = 'Corrigez la configuration du nœud, puis testez à nouveau le workflow.',
	action,
}: {
	message: string
	detail?: string
	hint?: string
	action?: ReactNode
}) {
	return (
		<div
			role="alert"
			className="rounded-md border border-state-alert/60 bg-state-alert/10 p-4"
		>
			<p className="flex items-center gap-2 font-semibold text-sm text-state-alert">
				<TriangleAlert className="size-4" aria-hidden />
				Erreur dans ce nœud
			</p>
			<p className="mt-2 font-medium text-base">{message}</p>
			{detail && (
				<code className="mt-2 block rounded-sm bg-background/70 px-2 py-1.5 font-mono text-muted-foreground text-xs">
					{detail}
				</code>
			)}
			<p className="mt-3 text-muted-foreground text-sm">{hint}</p>
			{action && <div className="mt-3">{action}</div>}
		</div>
	)
}
