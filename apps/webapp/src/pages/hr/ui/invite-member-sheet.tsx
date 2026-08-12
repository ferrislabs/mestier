import { Check, Copy, Loader2 } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { buildInviteLink } from '#/modules/invite-path'

export interface InviteMemberSheetProps {
	open: boolean
	memberName: string
	/** `null` before the link is generated, or once the sheet has been
	 * closed — the backend never returns the clear token a second time, so
	 * there is nothing to restore here even if it wanted to. */
	token: string | null
	isGenerating: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onGenerate: () => void
}

/**
 * Pure presentation — all state (including the token itself) lives in the
 * feature. The one local `useState` here (`copied`) is transient UI
 * feedback, not business data, so it stays.
 */
export function InviteMemberSheet({
	open,
	memberName,
	token,
	isGenerating,
	error,
	onOpenChange,
	onGenerate,
}: InviteMemberSheetProps) {
	const [copied, setCopied] = useState(false)
	const link = token ? buildInviteLink(token) : ''

	return (
		<Sheet
			open={open}
			onOpenChange={(next) => {
				setCopied(false)
				onOpenChange(next)
			}}
		>
			<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-lg">
				<SheetHeader className="border-b">
					<SheetTitle>Inviter {memberName}</SheetTitle>
					<SheetDescription>
						Ce lien donne accès au siège de {memberName}. Il n’est valable
						qu’une fois et affiché une seule fois — copiez-le avant de fermer
						cette fenêtre.
					</SheetDescription>
				</SheetHeader>

				<div className="flex-1 space-y-4 p-4">
					{token ? (
						<div className="flex flex-col gap-2">
							<Label htmlFor="invite-link">Lien d’invitation</Label>
							<div className="flex gap-2">
								<Input id="invite-link" readOnly value={link} />
								<Button
									type="button"
									variant="outline"
									onClick={() => {
										void navigator.clipboard.writeText(link)
										setCopied(true)
									}}
								>
									{copied ? <Check /> : <Copy />}
									{copied ? 'Copié' : 'Copier'}
								</Button>
							</div>
							<p className="text-xs text-muted-foreground">
								Ce lien ne sera plus jamais affiché après la fermeture de cette
								fenêtre.
							</p>
						</div>
					) : (
						<Button type="button" onClick={onGenerate} disabled={isGenerating}>
							{isGenerating ? <Loader2 className="animate-spin" /> : null}
							Générer le lien
						</Button>
					)}

					{error ? (
						<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
							{error}
						</p>
					) : null}
				</div>

				<SheetFooter className="border-t bg-background">
					<Button
						type="button"
						variant="ghost"
						onClick={() => onOpenChange(false)}
					>
						Fermer
					</Button>
				</SheetFooter>
			</SheetContent>
		</Sheet>
	)
}
