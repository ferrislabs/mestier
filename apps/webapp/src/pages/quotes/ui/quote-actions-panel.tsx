import { Link } from '@tanstack/react-router'
import {
	ArrowRightLeft,
	CircleCheck,
	CircleX,
	Copy,
	Download,
	GitBranch,
	Loader2,
	type LucideIcon,
	Receipt,
	RotateCcw,
	Save,
	Send,
	Trash2,
	TriangleAlert,
} from 'lucide-react'
import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { FormField } from '#/components/form-field'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Textarea } from '#/components/ui/textarea'
import type { QuoteStatus } from '#/hooks/use-quotes'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	availableQuoteActions,
	type QuoteAction,
} from '#/pages/quotes/lib/quote-actions'
import { invoiceNewPath } from '#/pages/quotes/lib/quote-invoicing'
import type { QuoteDisplayStatus } from '#/pages/quotes/lib/quote-status'

type Dialog = 'send' | 'accept' | 'decline' | 'cancel' | 'reopen' | 'delete'

const LABELS: Record<Exclude<QuoteAction, 'project' | 'invoice'>, string> = {
	send: 'Envoyer au client',
	accept: 'Marquer comme accepté',
	decline: 'Marquer comme refusé',
	cancel: 'Annuler le devis',
	reopen: 'Repasser en brouillon',
	revise: 'Créer une révision',
	duplicate: 'Dupliquer',
	delete: 'Supprimer',
}

const ICONS: Record<Exclude<QuoteAction, 'project' | 'invoice'>, LucideIcon> = {
	send: Send,
	accept: CircleCheck,
	decline: CircleX,
	cancel: CircleX,
	reopen: RotateCcw,
	revise: GitBranch,
	duplicate: Copy,
	delete: Trash2,
}

interface QuoteActionsPanelProps {
	organizationSlug: string
	quoteId: string
	status: QuoteStatus
	displayStatus: QuoteDisplayStatus
	label: string
	validityNote: string | null
	defaultSigner: string
	canSave: boolean
	canSend: boolean
	identityIncompleteForSending: boolean
	isSaving: boolean
	isDeleting: boolean
	isDuplicating: boolean
	isRevising: boolean
	isDownloading: boolean
	onSave: () => void
	onTransition: (status: QuoteStatus) => void
	onAccept: (signer: string) => void
	onDecline: (reason: string) => void
	onRevise: () => void
	onDelete: () => void
	onDuplicate: () => void
	onDownloadPdf: () => void
}

export function QuoteActionsPanel({
	organizationSlug,
	quoteId,
	status,
	displayStatus,
	label,
	validityNote,
	defaultSigner,
	canSave,
	canSend,
	identityIncompleteForSending,
	isSaving,
	isDeleting,
	isDuplicating,
	isRevising,
	isDownloading,
	onSave,
	onTransition,
	onAccept,
	onDecline,
	onRevise,
	onDelete,
	onDuplicate,
	onDownloadPdf,
}: QuoteActionsPanelProps) {
	const [dialog, setDialog] = useState<Dialog | null>(null)
	const [showErrors, setShowErrors] = useState(false)
	const [signer, setSigner] = useState(defaultSigner)
	const [reason, setReason] = useState('')
	const actions = availableQuoteActions(displayStatus)
	const busy = isSaving || isDeleting || isDuplicating || isRevising
	const draft = status === 'DRAFT'

	const close = () => setDialog(null)

	function trigger(action: Exclude<QuoteAction, 'project' | 'invoice'>) {
		if (action === 'duplicate') return onDuplicate()
		if (action === 'revise') return onRevise()
		if (action === 'accept') setSigner(defaultSigner)
		if (action === 'decline') setReason('')
		if (action === 'send' && !canSend) return setShowErrors(true)
		setDialog(action)
	}

	return (
		<RequirePermission permission="MANAGE_QUOTES">
			<div className="grid gap-3">
				<Button
					type="button"
					variant="outline"
					className="w-full justify-start"
					disabled={!canSave || busy}
					onClick={onSave}
				>
					{isSaving ? (
						<Loader2 className="animate-spin" />
					) : (
						<Save aria-hidden />
					)}
					Enregistrer
				</Button>
				{displayStatus === 'expired' && (
					<p className="text-muted-foreground text-sm">
						{validityNote ?? 'Ce devis a expiré'} : il ne peut plus être
						accepté. Crée une révision pour proposer de nouvelles conditions.
					</p>
				)}
				{actions.map((action) => {
					if (action === 'invoice')
						return (
							<Button
								key={action}
								variant="outline"
								className="w-full justify-start"
								asChild
							>
								<Link to={invoiceNewPath(organizationSlug, quoteId)}>
									<Receipt aria-hidden />
									Facturer
								</Link>
							</Button>
						)
					if (action === 'project')
						return (
							<Button
								key={action}
								variant="default"
								className="w-full justify-start"
								asChild
							>
								<Link
									to={buildOrgPath(
										organizationSlug,
										`/crm/devis/${quoteId}/passation`,
									)}
								>
									<ArrowRightLeft aria-hidden />
									Transformer en projet
								</Link>
							</Button>
						)
					const Icon = ICONS[action]
					const destructive = action === 'delete' || action === 'cancel'
					return (
						<Button
							key={action}
							type="button"
							variant={action === 'send' ? 'default' : 'outline'}
							className={cn(
								'w-full justify-start',
								destructive && 'text-destructive hover:text-destructive',
							)}
							disabled={
								busy ||
								(action === 'send' && identityIncompleteForSending) ||
								(action === 'duplicate' && !canSave)
							}
							onClick={() => trigger(action)}
						>
							<Icon aria-hidden />
							{action === 'delete' && draft
								? 'Supprimer le brouillon'
								: LABELS[action]}
						</Button>
					)
				})}
				<Button
					type="button"
					variant="outline"
					className="w-full justify-start"
					disabled={isDownloading}
					onClick={onDownloadPdf}
				>
					{isDownloading ? (
						<Loader2 className="animate-spin" />
					) : (
						<Download aria-hidden />
					)}
					Télécharger le PDF
				</Button>
				{showErrors && !canSend && (
					<div
						role="alert"
						className="flex items-start gap-2.5 rounded-lg border border-state-today/35 bg-state-today/10 px-3 py-2.5 text-sm"
					>
						<TriangleAlert
							className="mt-0.5 size-4 shrink-0 text-state-today"
							aria-hidden
						/>
						<p>
							Le devis n’est pas prêt à être envoyé : renseigne le client, son
							adresse, l’objet et des lignes complètes.
						</p>
					</div>
				)}
				<ConfirmDialog
					open={dialog === 'send'}
					onOpenChange={(open) => !open && close()}
					title="Envoyer le devis au client ?"
					description="Un numéro sera attribué et le devis passera au statut envoyé."
					confirmLabel="Envoyer au client"
					onConfirm={() => {
						onTransition('SENT')
						setShowErrors(false)
						close()
					}}
				/>
				<ConfirmDialog
					open={dialog === 'accept'}
					onOpenChange={(open) => !open && close()}
					title={`Marquer le devis ${label} comme accepté`}
					description="Le client a donné son accord. Le devis pourra ensuite être transformé en projet et facturé."
					confirmLabel="Marquer comme accepté"
					disabled={signer.trim() === ''}
					onConfirm={() => {
						onAccept(signer.trim())
						close()
					}}
				>
					<FormField
						id={`accept-${quoteId}-signer`}
						label="Nom de la personne qui signe"
						required
					>
						<Input
							id={`accept-${quoteId}-signer`}
							value={signer}
							autoComplete="off"
							onChange={(event) => setSigner(event.target.value)}
						/>
					</FormField>
				</ConfirmDialog>
				<ConfirmDialog
					open={dialog === 'decline'}
					onOpenChange={(open) => !open && close()}
					title={`Marquer le devis ${label} comme refusé`}
					description="Tu pourras ensuite créer une révision pour proposer de nouvelles conditions."
					confirmLabel="Marquer comme refusé"
					destructive
					onConfirm={() => {
						onDecline(reason.trim())
						close()
					}}
				>
					<FormField
						id={`decline-${quoteId}-reason`}
						label="Motif du refus"
						hint="Facultatif : prix, délai, concurrent retenu…"
					>
						<Textarea
							id={`decline-${quoteId}-reason`}
							rows={3}
							value={reason}
							onChange={(event) => setReason(event.target.value)}
						/>
					</FormField>
				</ConfirmDialog>
				<ConfirmDialog
					open={dialog === 'cancel'}
					onOpenChange={(open) => !open && close()}
					title="Annuler ce devis ?"
					description="Le devis est abandonné. Il reste consultable dans la liste."
					confirmLabel="Annuler le devis"
					destructive
					onConfirm={() => {
						onTransition('CANCELLED')
						close()
					}}
				/>
				<ConfirmDialog
					open={dialog === 'reopen'}
					onOpenChange={(open) => !open && close()}
					title="Repasser le devis en brouillon ?"
					description="Le devis redevient un brouillon que tu peux modifier librement."
					confirmLabel="Repasser en brouillon"
					onConfirm={() => {
						onTransition('DRAFT')
						close()
					}}
				/>
				<ConfirmDialog
					open={dialog === 'delete'}
					onOpenChange={(open) => !open && close()}
					title={draft ? 'Supprimer ce brouillon ?' : 'Supprimer ce devis ?'}
					description={
						draft
							? 'Le brouillon sera supprimé définitivement.'
							: `Le devis ${label} sera supprimé de la liste. Cette action est irréversible.`
					}
					confirmLabel={draft ? 'Supprimer le brouillon' : 'Supprimer le devis'}
					destructive
					onConfirm={() => {
						onDelete()
						close()
					}}
				/>
			</div>
		</RequirePermission>
	)
}
