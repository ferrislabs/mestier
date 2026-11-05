export type ExpressionPreviewState =
	| { status: 'idle' }
	| { status: 'loading' }
	| { status: 'resolved'; value: unknown }
	| { status: 'error'; message: string }

export interface ExpressionPreviewProps {
	state: ExpressionPreviewState
}

function display(value: unknown): string {
	return typeof value === 'string' ? value : JSON.stringify(value)
}

export function ExpressionPreview({ state }: ExpressionPreviewProps) {
	if (state.status === 'idle') return null

	if (state.status === 'error') {
		return (
			<p role="alert" className="truncate text-state-alert text-xs">
				{state.message}
			</p>
		)
	}

	return (
		<p className="truncate font-mono text-muted-foreground text-xs">
			Aperçu : {state.status === 'loading' ? '…' : display(state.value)}
		</p>
	)
}
