import {
	Ban,
	Banknote,
	BellRing,
	Download,
	FileCheck,
	FileMinus,
	Info,
	Loader2,
	Save,
	Send,
} from 'lucide-react'
import { type ReactNode, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import type { Invoice } from '#/hooks/use-invoices'
import type { creditNoteLines } from '#/pages/invoices/lib/credit-note-request'
import {
	ACTION_LABELS,
	type ActionState,
	type InvoiceActions,
	unavailableNotes,
} from '#/pages/invoices/lib/invoice-actions'
import type { PaymentValues } from '#/pages/invoices/lib/payment-form'
import { CreditNoteDialog } from '#/pages/invoices/ui/credit-note-dialog'
import { IssueDialog } from '#/pages/invoices/ui/issue-dialog'
import { PaymentDialog } from '#/pages/invoices/ui/payment-dialog'

type DialogKind = 'issue' | 'payment' | 'credit' | 'cancel'

interface InvoiceActionsBarProps {
	invoice: Invoice
	actions: InvoiceActions
	remainingCents: number | null
	isDirty: boolean
	canSave: boolean
	isSaving: boolean
	isIssuing: boolean
	isCancelling: boolean
	isRecordingPayment: boolean
	isIssuingCreditNote: boolean
	isSending: boolean
	isReminding: boolean
	isDownloading: boolean
	issueError: string | null
	paymentError: string | null
	creditNoteError: string | null
	onSave: () => void
	onIssue: () => Promise<boolean>
	onCancel: () => Promise<boolean>
	onRecordPayment: (values: PaymentValues) => Promise<boolean>
	onIssueCreditNote: (request: {
		reason: string
		lines: ReturnType<typeof creditNoteLines>
	}) => Promise<boolean>
	onSend: () => Promise<string>
	onRemind: () => Promise<string>
	onDownloadPdf: () => void
}

export function InvoiceActionsBar({
	invoice,
	actions,
	remainingCents,
	isDirty,
	canSave,
	isSaving,
	isIssuing,
	isCancelling,
	isRecordingPayment,
	isIssuingCreditNote,
	isSending,
	isReminding,
	isDownloading,
	issueError,
	paymentError,
	creditNoteError,
	onSave,
	onIssue,
	onCancel,
	onRecordPayment,
	onIssueCreditNote,
	onSend,
	onRemind,
	onDownloadPdf,
}: InvoiceActionsBarProps) {
	const [dialog, setDialog] = useState<DialogKind | null>(null)
	const [notice, setNotice] = useState<string | null>(null)
	const notes = unavailableNotes(actions)
	const draft = invoice.status === 'DRAFT'

	const close = (kind: DialogKind) => (open: boolean) =>
		setDialog(open ? kind : null)

	function button(
		action: ActionState,
		onClick: () => void,
		icon: ReactNode,
		label: string,
		variant: 'default' | 'outline' = 'outline',
		className?: string,
		disabled?: boolean,
	) {
		if (!action.visible) return null
		return (
			<Button
				type="button"
				size="sm"
				variant={variant}
				disabled={!action.enabled || disabled}
				onClick={onClick}
				className={className}
			>
				{icon}
				{label}
			</Button>
		)
	}

	return (
		<div className="grid gap-2">
			<RequirePermission permission="MANAGE_INVOICES">
				<div className="flex flex-wrap items-center gap-2">
					{draft && (
						<Button
							type="button"
							size="sm"
							variant="outline"
							disabled={!canSave || isSaving}
							onClick={onSave}
						>
							{isSaving ? (
								<Loader2 className="animate-spin" />
							) : (
								<Save aria-hidden />
							)}
							Enregistrer
						</Button>
					)}
					{button(
						actions.issue,
						() => setDialog('issue'),
						<FileCheck aria-hidden />,
						ACTION_LABELS.issue,
						'default',
					)}
					{button(
						actions.recordPayment,
						() => setDialog('payment'),
						<Banknote aria-hidden />,
						ACTION_LABELS.recordPayment,
						actions.recordPayment.enabled ? 'default' : 'outline',
					)}
					{button(
						actions.send,
						() => void onSend().then(setNotice),
						isSending ? (
							<Loader2 className="animate-spin" />
						) : (
							<Send aria-hidden />
						),
						actions.sendLabel,
						'outline',
						undefined,
						isSending,
					)}
					{button(
						actions.remind,
						() => void onRemind().then(setNotice),
						isReminding ? (
							<Loader2 className="animate-spin" />
						) : (
							<BellRing aria-hidden />
						),
						ACTION_LABELS.remind,
						'outline',
						undefined,
						isReminding,
					)}
					{button(
						actions.creditNote,
						() => setDialog('credit'),
						<FileMinus aria-hidden />,
						ACTION_LABELS.creditNote,
					)}
					{button(
						actions.cancel,
						() => setDialog('cancel'),
						<Ban aria-hidden />,
						ACTION_LABELS.cancel,
						'outline',
						'text-destructive hover:text-destructive',
						isCancelling,
					)}
					<Button
						type="button"
						size="sm"
						variant="outline"
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
				</div>
			</RequirePermission>
			<output className="block text-sm text-state-progress empty:hidden">
				{notice}
			</output>
			{notes.length > 0 && (
				<ul className="grid gap-1 text-muted-foreground text-xs">
					{notes.map((note) => (
						<li key={note.key} className="flex items-start gap-1.5">
							<Info className="mt-0.5 size-3.5 shrink-0" aria-hidden />
							<span>
								<span className="font-medium text-foreground">
									{note.label}
								</span>{' '}
								est indisponible : {note.reason}
							</span>
						</li>
					))}
				</ul>
			)}
			{draft && (
				<IssueDialog
					open={dialog === 'issue'}
					onOpenChange={close('issue')}
					invoice={invoice}
					isDirty={isDirty}
					isPending={isIssuing}
					error={issueError}
					onIssue={onIssue}
				/>
			)}
			{invoice.status !== 'DRAFT' && invoice.status !== 'CANCELLED' && (
				<>
					<PaymentDialog
						open={dialog === 'payment'}
						onOpenChange={close('payment')}
						remainingCents={remainingCents}
						isPending={isRecordingPayment}
						error={paymentError}
						onSubmit={onRecordPayment}
					/>
					<CreditNoteDialog
						open={dialog === 'credit'}
						onOpenChange={close('credit')}
						invoice={invoice}
						isPending={isIssuingCreditNote}
						error={creditNoteError}
						onSubmit={onIssueCreditNote}
					/>
				</>
			)}
			<ConfirmDialog
				open={dialog === 'cancel'}
				onOpenChange={close('cancel')}
				title="Annuler cette facture ?"
				description={`La facture ${invoice.number ?? 'en brouillon'} sera annulée. Cette action est irréversible.`}
				confirmLabel="Annuler la facture"
				destructive
				onConfirm={() => {
					void onCancel().then((done) => {
						if (done) setDialog(null)
					})
				}}
			/>
		</div>
	)
}
