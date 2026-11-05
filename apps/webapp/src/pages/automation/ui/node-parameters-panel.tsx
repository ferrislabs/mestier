import { type ReactNode, useId } from 'react'
import { Label } from '#/components/ui/label'
import type { HttpMethod } from '#/pages/automation/lib/collections'
import { MethodBadge } from '#/pages/automation/ui/method-badge'
import { NdvPanel } from '#/pages/automation/ui/ndv-panel'

export interface OperationSummary {
	method: HttpMethod
	path: string
	collectionTitle: string
	baseUrl: string
}

export function OperationField({
	operation,
}: {
	operation: OperationSummary | null
}) {
	const controlId = useId()
	return (
		<div className="grid gap-2">
			<Label htmlFor={controlId}>Opération</Label>
			{operation ? (
				<div
					id={controlId}
					className="flex items-center gap-3 rounded-md border border-border bg-card px-3 py-2"
				>
					<MethodBadge method={operation.method} />
					<span className="min-w-0 flex-1">
						<span className="block truncate font-mono text-sm">
							{operation.path}
						</span>
						<span className="block truncate text-muted-foreground text-xs">
							{operation.collectionTitle}
							{operation.baseUrl ? ` · ${operation.baseUrl}` : ''}
						</span>
					</span>
				</div>
			) : (
				<p id={controlId} className="text-sm text-state-alert">
					Opération introuvable : la collection a été supprimée ou modifiée.
				</p>
			)}
		</div>
	)
}

const EXPRESSION_EXAMPLE = '{{ trigger.client.nom }}'

export function NodeParametersPanel({
	acceptsExpressions,
	children,
}: {
	acceptsExpressions: boolean
	children: ReactNode
}) {
	return (
		<NdvPanel title="Paramètres" tone="center">
			<form className="grid gap-4" onSubmit={(event) => event.preventDefault()}>
				{children}
				{acceptsExpressions && (
					<p className="rounded-md bg-muted px-3 py-2 text-muted-foreground text-xs">
						Astuce : glissez une variable depuis la colonne Entrée, ou écrivez
						une expression, par exemple{' '}
						<code className="font-mono text-foreground">
							{EXPRESSION_EXAMPLE}
						</code>
						.
					</p>
				)}
			</form>
		</NdvPanel>
	)
}
