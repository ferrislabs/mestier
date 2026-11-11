import { Check, Loader2 } from 'lucide-react'
import type { SaveState } from '#/pages/knowledge-documents/lib/document-session'

export const SAVING_LABEL = 'Enregistrement…'
export const SAVED_LABEL = 'Enregistré'

interface SaveIndicatorProps {
	state: SaveState
	error: string | null
}

export function SaveIndicator({ state, error }: SaveIndicatorProps) {
	if (state === 'error') {
		return (
			<output className="text-[13px] text-state-alert">
				{error ?? 'Enregistrement impossible.'}
			</output>
		)
	}
	if (state === 'saving' || state === 'dirty') {
		return (
			<output className="flex items-center gap-1.5 text-[13px] text-muted-foreground">
				<Loader2 aria-hidden className="size-3.5 animate-spin" />
				{SAVING_LABEL}
			</output>
		)
	}
	if (state === 'saved') {
		return (
			<output className="flex items-center gap-1.5 text-[13px] text-muted-foreground">
				<Check aria-hidden className="size-3.5" />
				{SAVED_LABEL}
			</output>
		)
	}
	return null
}
