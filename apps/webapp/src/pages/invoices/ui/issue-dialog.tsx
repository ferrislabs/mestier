import { Link } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Invoice } from '#/hooks/use-invoices'
import { buildOrgPath } from '#/modules/org-path'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import { LockBanner, Money } from '#/pages/quotes/ui/doc/primitives'
import { LEGAL_IDENTITY_FIELD_LABELS } from '#/pages/settings/types'

interface IssueDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	invoice: Invoice
	isDirty: boolean
	isPending: boolean
	error: string | null
	onIssue: () => Promise<boolean>
}

export function IssueDialog({
	open,
	onOpenChange,
	invoice,
	isDirty,
	isPending,
	error,
	onIssue,
}: IssueDialogProps) {
	const { activeOrganization } = useActiveOrganization()
	const missing = activeOrganization.missing_legal_identity_fields

	const confirm = async () => {
		if (await onIssue()) onOpenChange(false)
	}

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-xl">
				<DialogHeader>
					<DialogTitle>Émettre la facture</DialogTitle>
					<DialogDescription>
						Vérifie les informations : l’émission attribue un numéro définitif
						et fige la facture.
					</DialogDescription>
				</DialogHeader>
				<div className="grid gap-4">
					{isDirty && (
						<div
							role="alert"
							className="rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-sm"
						>
							<p className="font-medium text-state-alert">
								Corrige ce point avant d’émettre
							</p>
							<p className="mt-1.5">
								Des modifications ne sont pas enregistrées : enregistre la
								facture pour qu’elles figurent sur le document émis.
							</p>
						</div>
					)}
					{missing.length > 0 && (
						<LockBanner tone="warn" title="Identité légale incomplète">
							<p>
								Il manque{' '}
								{missing
									.map((field) => LEGAL_IDENTITY_FIELD_LABELS[field] ?? field)
									.join(', ')}
								. L’émission peut être refusée.{' '}
								<Link
									to={buildOrgPath(activeOrganization.slug, '/settings')}
									hash="organisation"
									className="rounded-sm font-medium underline underline-offset-2 outline-none focus-visible:ring-2 focus-visible:ring-ring"
								>
									Ouvrir les paramètres
								</Link>
							</p>
						</LockBanner>
					)}
					<dl className="grid gap-2 rounded-lg border bg-muted/30 p-4 text-sm">
						<div className="flex justify-between gap-4">
							<dt className="text-muted-foreground">Numéro attribué</dt>
							<dd className="font-semibold">À l’émission</dd>
						</div>
						<div className="flex justify-between gap-4">
							<dt className="text-muted-foreground">Échéance</dt>
							<dd>{formatDocumentDate(invoice.due_at)}</dd>
						</div>
						<div className="flex justify-between gap-4">
							<dt className="text-muted-foreground">Total TTC</dt>
							<dd className="font-semibold">
								<Money cents={invoice.gross_cents} />
							</dd>
						</div>
					</dl>
					<LockBanner tone="warn" title="Émission définitive">
						Une fois émise, la facture ne pourra plus être modifiée. Pour la
						corriger, tu devras émettre un avoir.
					</LockBanner>
					{error && (
						<p role="alert" className="text-sm text-state-alert">
							{error}
						</p>
					)}
				</div>
				<DialogFooter>
					<Button
						type="button"
						variant="outline"
						onClick={() => onOpenChange(false)}
					>
						Annuler
					</Button>
					<Button
						type="button"
						disabled={isDirty || isPending}
						onClick={() => void confirm()}
					>
						{isPending ? <Loader2 className="animate-spin" /> : null}
						Émettre la facture
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
