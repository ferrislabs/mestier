import { Check, Copy } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { formatDate } from '#/pages/cursus/participants/lib/forms'

export interface InvitationLinkDialogProps {
	participantName: string
	link: string | null
	expiresAt: string | null
	onClose: () => void
}

export function InvitationLinkDialog({
	participantName,
	link,
	expiresAt,
	onClose,
}: InvitationLinkDialogProps) {
	const [copied, setCopied] = useState(false)

	return (
		<Dialog
			open={link !== null}
			onOpenChange={(open) => {
				if (!open) {
					setCopied(false)
					onClose()
				}
			}}
		>
			<DialogContent className="sm:max-w-lg">
				<DialogHeader>
					<DialogTitle>Invitation de {participantName}</DialogTitle>
					<DialogDescription>
						Ce lien n'est affiché qu'une seule fois. Copiez-le avant de fermer
						cette fenêtre.
					</DialogDescription>
				</DialogHeader>
				<div className="flex flex-col gap-2">
					<Label htmlFor="invitation-link">Lien d'invitation</Label>
					<div className="flex gap-2">
						<Input id="invitation-link" readOnly value={link ?? ''} />
						<Button
							type="button"
							variant="outline"
							onClick={() => {
								void navigator.clipboard.writeText(link ?? '')
								setCopied(true)
							}}
						>
							{copied ? <Check /> : <Copy />}
							{copied ? 'Copié' : 'Copier'}
						</Button>
					</div>
					<p className="text-sm text-muted-foreground">
						Valable jusqu'au {formatDate(expiresAt)}.
					</p>
				</div>
				<DialogFooter>
					<Button type="button" variant="ghost" onClick={onClose}>
						Fermer
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
