import { Link } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useId, useState } from 'react'
import { FormField } from '#/components/form-field'
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
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Invoice } from '#/hooks/use-invoices'
import { buildOrgPath } from '#/modules/org-path'
import { issueDateErrors } from '#/pages/invoices/lib/invoice-issue'
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
	bounds: { min: string | null; max: string }
	onIssue: (issueDate: string) => Promise<boolean>
}

function IssueForm({
	invoice,
	isDirty,
	isPending,
	error,
	bounds,
	onIssue,
	onClose,
}: Omit<IssueDialogProps, 'open' | 'onOpenChange'> & { onClose: () => void }) {
	const { activeOrganization } = useActiveOrganization()
	const baseId = useId()
	const missing = activeOrganization.missing_legal_identity_fields
	const [issueDate, setIssueDate] = useState(bounds.max)
	const dateErrors = issueDateErrors(issueDate, bounds.max, bounds.min)

	const confirm = async () => {
		if (dateErrors.length > 0) return
		if (await onIssue(issueDate)) onClose()
	}

	return (
		<>
			<DialogHeader>
				<DialogTitle>Émettre la facture</DialogTitle>
				<DialogDescription>
					Vérifie les informations : l’émission attribue un numéro définitif et
					fige la facture.
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
							Des modifications ne sont pas enregistrées : enregistre la facture
							pour qu’elles figurent sur le document émis.
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
				<FormField
					id={`${baseId}-date`}
					label="Date d’émission"
					required
					hint={
						bounds.min === null
							? 'Pas plus tard qu’aujourd’hui. Date indicative : le serveur horodate l’émission.'
							: `Entre le ${formatDocumentDate(bounds.min)} (dernière facture émise) et aujourd’hui : la numérotation reste chronologique. Date indicative : le serveur horodate l’émission.`
					}
				>
					<Input
						id={`${baseId}-date`}
						type="date"
						value={issueDate}
						min={bounds.min ?? undefined}
						max={bounds.max}
						onChange={(event) => setIssueDate(event.target.value)}
						className="w-44"
					/>
				</FormField>
				{dateErrors.length > 0 && (
					<div
						role="alert"
						className="rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-sm"
					>
						<p className="font-medium text-state-alert">
							Corrige ces points avant d’émettre
						</p>
						<ul className="mt-1.5 grid gap-1.5">
							{dateErrors.map((message) => (
								<li key={message}>{message}</li>
							))}
						</ul>
					</div>
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
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button
					type="button"
					disabled={isDirty || isPending || dateErrors.length > 0}
					onClick={() => void confirm()}
				>
					{isPending ? <Loader2 className="animate-spin" /> : null}
					Émettre la facture
				</Button>
			</DialogFooter>
		</>
	)
}

export function IssueDialog({ open, onOpenChange, ...rest }: IssueDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-xl">
				<IssueForm {...rest} onClose={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}
