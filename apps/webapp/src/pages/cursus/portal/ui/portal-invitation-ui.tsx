import { AlertCircle, Loader2 } from 'lucide-react'
import { PageShell } from '#/components/ui/surface'

interface PortalInvitationUIProps {
	errorMessage: string | null
}

export function PortalInvitationUI({ errorMessage }: PortalInvitationUIProps) {
	return (
		<PageShell className="max-w-md items-center py-24 text-center">
			{errorMessage ? (
				<>
					<AlertCircle className="size-8 text-destructive" />
					<p role="alert" className="font-medium">
						{errorMessage}
					</p>
				</>
			) : (
				<>
					<Loader2 className="size-8 animate-spin text-primary" />
					<p className="font-medium">Acceptation de l’invitation…</p>
				</>
			)}
		</PageShell>
	)
}
