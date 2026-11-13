import { Check, CircleAlert } from 'lucide-react'
import type { SaveState } from '#/pages/knowledge-documents/lib/document-session'

export const SAVING_LABEL = 'Enregistrement…'
export const SAVED_LABEL = 'Enregistré'

interface SaveIndicatorProps {
	state: SaveState
	error: string | null
}

export function SaveIndicator({ state, error }: SaveIndicatorProps) {
	if (state === 'error') {
		const message = error ?? 'Enregistrement impossible.'
		return (
			<output
				title={message}
				className="flex min-w-0 max-w-64 items-center gap-1.5 text-[13px] text-state-alert"
			>
				<CircleAlert aria-hidden className="size-3.5 shrink-0" />
				<span className="truncate">{message}</span>
			</output>
		)
	}
	if (state === 'saving' || state === 'dirty') {
		return (
			<output className="text-[13px] text-muted-foreground">
				{SAVING_LABEL}
			</output>
		)
	}
	if (state === 'saved') {
		return (
			<output className="flex items-center gap-1 text-[13px] text-muted-foreground">
				<Check aria-hidden className="size-3.5" />
				{SAVED_LABEL}
			</output>
		)
	}
	return null
}
