import { AlertCircle, Loader2 } from 'lucide-react'
import { FullscreenMessage } from '#/components/org-gate'
import { Button } from '#/components/ui/button'

export type InviteAcceptStatus = 'pending' | 'error'

export interface InviteAcceptUIProps {
	status: InviteAcceptStatus
	/** Set only when `status === 'error'`. */
	errorMessage: string | null
}

/**
 * Pure presentation — reuses `FullscreenMessage` from `org-gate.tsx`, the
 * same full-page layout `AuthGate`/`OrgGate` already use for "loading" and
 * "error" states, so this screen reads as part of the same auth sequence
 * rather than a visually distinct page.
 */
export function InviteAcceptUI({ status, errorMessage }: InviteAcceptUIProps) {
	if (status === 'error') {
		return (
			<FullscreenMessage
				icon={<AlertCircle className="size-8 text-destructive" />}
				title="Invitation impossible à accepter"
				message={errorMessage ?? 'Une erreur est survenue.'}
				action={
					<Button asChild>
						<a href="/">Retour à l’accueil</a>
					</Button>
				}
			/>
		)
	}

	return (
		<FullscreenMessage
			icon={<Loader2 className="size-8 animate-spin text-primary" />}
			title="Acceptation de l’invitation…"
			message="Un instant, nous vous ajoutons à l’organisation."
		/>
	)
}
