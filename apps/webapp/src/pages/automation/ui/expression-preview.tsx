export type ExpressionPreviewState =
	| { status: 'idle' }
	| { status: 'loading' }
	| { status: 'resolved'; value: unknown }
	| { status: 'error'; message: string }

export interface ExpressionPreviewProps {
	fieldLabel: string | null
	state: ExpressionPreviewState
}

export function ExpressionPreview({
	fieldLabel,
	state,
}: ExpressionPreviewProps) {
	if (fieldLabel === null) return null

	return (
		<div className="flex flex-col gap-1 rounded-md border bg-muted/40 p-2 text-xs">
			<p className="font-medium text-muted-foreground">
				Aperçu — {fieldLabel}
			</p>
			{state.status === 'idle' ? (
				<p className="text-muted-foreground">
					Cliquez ou glissez une donnée pour l’insérer.
				</p>
			) : null}
			{state.status === 'loading' ? (
				<p className="text-muted-foreground">Évaluation en cours…</p>
			) : null}
			{state.status === 'resolved' ? (
				<code className="whitespace-pre-wrap break-all font-mono">
					{JSON.stringify(state.value)}
				</code>
			) : null}
			{state.status === 'error' ? (
				<p role="alert" className="text-destructive">
					{state.message}
				</p>
			) : null}
		</div>
	)
}
