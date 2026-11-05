import { Loader2 } from 'lucide-react'
import { type FormEvent, useId, useState } from 'react'
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
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import type { Invoice } from '#/hooks/use-invoices'
import {
	type CreditMode,
	creditedQuantity,
	creditLineIssue,
	creditNoteErrors,
	creditNoteLines,
	creditPreviewNetCents,
	fillAll,
	type QuantityInputs,
	selectedCount,
} from '#/pages/invoices/lib/credit-note-request'
import {
	AutoTextarea,
	Money,
	OptionCard,
} from '#/pages/quotes/ui/doc/primitives'

interface CreditNoteDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	invoice: Invoice
	isPending: boolean
	error: string | null
	onSubmit: (request: {
		reason: string
		lines: ReturnType<typeof creditNoteLines>
	}) => Promise<boolean>
}

function CreditForm({
	invoice,
	isPending,
	error,
	onClose,
	onSubmit,
}: Omit<CreditNoteDialogProps, 'open' | 'onOpenChange'> & {
	onClose: () => void
}) {
	const baseId = useId()
	const [mode, setMode] = useState<CreditMode>('partial')
	const [reason, setReason] = useState('')
	const [inputs, setInputs] = useState<QuantityInputs>({})
	const [attempted, setAttempted] = useState(false)
	const lines = invoice.lines
	const errors = creditNoteErrors(lines, mode, inputs, reason)
	const netCents = creditPreviewNetCents(lines, mode, inputs)

	const submit = async (event: FormEvent) => {
		event.preventDefault()
		if (errors.length > 0) {
			setAttempted(true)
			return
		}
		const done = await onSubmit({
			reason: reason.trim(),
			lines: creditNoteLines(lines, mode, inputs),
		})
		if (done) onClose()
	}

	return (
		<form onSubmit={submit} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>Émettre un avoir</DialogTitle>
				<DialogDescription>
					Un avoir corrige la facture sans la modifier : la facture reste
					archivée telle qu’elle a été émise, et l’avoir en annule tout ou
					partie.
				</DialogDescription>
			</DialogHeader>
			<fieldset className="grid gap-3 sm:grid-cols-2">
				<legend className="mb-2 font-medium text-sm">Type d’avoir</legend>
				<OptionCard
					name={`${baseId}-mode`}
					value="total"
					checked={mode === 'total'}
					onSelect={() => setMode('total')}
					title="Annulation totale"
					description="Crédite toutes les lignes de la facture, en entier."
				/>
				<OptionCard
					name={`${baseId}-mode`}
					value="partial"
					checked={mode === 'partial'}
					onSelect={() => setMode('partial')}
					title="Correction partielle"
					description="Choisis la quantité à créditer, ligne par ligne."
				/>
			</fieldset>
			<FormField
				id={`${baseId}-reason`}
				label="Motif de l’avoir"
				required
				hint="Il figure sur l’avoir remis au client."
			>
				<AutoTextarea
					id={`${baseId}-reason`}
					value={reason}
					placeholder="Taux de TVA erroné, travaux non réalisés…"
					onChange={(event) => setReason(event.target.value)}
					className="min-h-16"
				/>
			</FormField>
			{mode === 'partial' && (
				<div className="flex flex-wrap items-center gap-2">
					<Button
						type="button"
						variant="outline"
						size="sm"
						onClick={() => setInputs(fillAll(lines))}
					>
						Tout sélectionner
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="sm"
						disabled={selectedCount(lines, mode, inputs) === 0}
						onClick={() => setInputs({})}
					>
						Tout effacer
					</Button>
				</div>
			)}
			<div className="overflow-x-auto rounded-lg border">
				<Table className="min-w-[36rem]">
					<caption className="sr-only">
						Quantités à créditer, ligne par ligne
					</caption>
					<TableHeader className="bg-muted/40">
						<TableRow className="hover:bg-transparent">
							<TableHead scope="col" className="h-9 w-10 text-xs">
								N°
							</TableHead>
							<TableHead scope="col" className="h-9 text-xs">
								Désignation
							</TableHead>
							<TableHead scope="col" className="h-9 text-right text-xs">
								Facturé
							</TableHead>
							<TableHead scope="col" className="h-9 w-40 text-right text-xs">
								À créditer
							</TableHead>
							<TableHead scope="col" className="h-9 text-right text-xs">
								Total HT
							</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{lines.map((line, index) => {
							const issue = creditLineIssue(line, mode, inputs)
							const quantity = creditedQuantity(line, mode, inputs)
							return (
								<TableRow key={line.id} className="align-top">
									<TableCell className="tabular-nums text-muted-foreground">
										{index + 1}
									</TableCell>
									<TableCell className="max-w-64 whitespace-normal">
										{line.label}
									</TableCell>
									<TableCell className="text-right tabular-nums">
										{line.quantity}
									</TableCell>
									<TableCell className="text-right">
										{mode === 'total' ? (
											<span className="tabular-nums">{line.quantity}</span>
										) : (
											<span className="flex items-center justify-end gap-1.5">
												<Button
													type="button"
													variant="ghost"
													size="xs"
													aria-label={`Créditer toute la ligne ${index + 1}`}
													onClick={() =>
														setInputs({ ...inputs, [line.id]: line.quantity })
													}
												>
													Tout
												</Button>
												<Input
													id={`${baseId}-qty-${line.id}`}
													inputMode="decimal"
													autoComplete="off"
													placeholder="0"
													aria-label={`Quantité à créditer, ligne ${index + 1}`}
													aria-invalid={issue !== null || undefined}
													value={inputs[line.id] ?? ''}
													onChange={(event) =>
														setInputs({
															...inputs,
															[line.id]: event.target.value,
														})
													}
													className="h-8 w-20 text-right tabular-nums"
												/>
											</span>
										)}
									</TableCell>
									<TableCell className="text-right">
										{quantity > 0 ? (
											<Money
												cents={creditPreviewNetCents([line], mode, inputs)}
											/>
										) : (
											<span className="text-muted-foreground">—</span>
										)}
									</TableCell>
								</TableRow>
							)
						})}
					</TableBody>
				</Table>
			</div>
			<output className="block rounded-lg border bg-muted/40 p-3 text-sm">
				<span className="flex items-baseline justify-between gap-4 font-semibold">
					<span>Total HT crédité</span>
					<Money cents={netCents} className="text-lg" />
				</span>
				<span className="mt-1 block text-muted-foreground">
					La TVA et le total TTC de l’avoir sont calculés à l’émission.
				</span>
			</output>
			{attempted && errors.length > 0 && (
				<ul
					role="alert"
					className="list-disc rounded-md bg-state-alert/8 py-2 pr-3 pl-7 text-[13px] text-state-alert"
				>
					{errors.map((message) => (
						<li key={message}>{message}</li>
					))}
				</ul>
			)}
			{error && (
				<p role="alert" className="text-sm text-state-alert">
					{error}
				</p>
			)}
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button type="submit" disabled={isPending}>
					{isPending ? <Loader2 className="animate-spin" /> : null}
					Émettre l’avoir
				</Button>
			</DialogFooter>
		</form>
	)
}

export function CreditNoteDialog({
	open,
	onOpenChange,
	...rest
}: CreditNoteDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-3xl">
				<CreditForm {...rest} onClose={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}
