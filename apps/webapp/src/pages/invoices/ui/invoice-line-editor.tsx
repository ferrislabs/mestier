import { ChevronRight, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { cn } from '#/lib/utils'
import {
	COMMON_VAT_RATES_BP,
	formatMoney,
	formatVatRateBp,
	type InvoiceLineFormValues,
	invoiceLineSummary,
	invoiceLineTotalCents,
} from '../types'

interface InvoiceLineEditorProps {
	index: number
	line: InvoiceLineFormValues
	isOpen: boolean
	canRemove: boolean
	/** The organization's VAT status, read once at the page level — an
	 * organization not subject to VAT gets no rate field at all, same rule
	 * as `QuoteLineEditor`. */
	vatEnabled: boolean
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<InvoiceLineFormValues>) => void
	onRemove: () => void
}

/**
 * One invoice line, folded to a summary until it is being edited — the
 * same collapse-one-at-a-time shape as `QuoteLineEditor`, minus the fields
 * an invoice line does not have (no unit, no catalogue link, no photos:
 * `InvoiceLineRequest` is only label/quantity/unit_price_cents/
 * vat_rate_basis_points).
 */
export function InvoiceLineEditor({
	index,
	line,
	isOpen,
	canRemove,
	vatEnabled,
	onOpenChange,
	onChange,
	onRemove,
}: InvoiceLineEditorProps) {
	const lineTotalCents = invoiceLineTotalCents(line)

	return (
		<Collapsible
			open={isOpen}
			onOpenChange={onOpenChange}
			className="bg-card data-[state=open]:bg-muted/20"
		>
			<div className="flex items-center gap-2 px-4 py-3">
				<CollapsibleTrigger className="flex min-w-0 flex-1 items-center gap-3 text-left">
					<ChevronRight
						className={cn(
							'size-4 shrink-0 text-muted-foreground transition-transform',
							isOpen && 'rotate-90',
						)}
					/>
					<span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-xs font-semibold text-muted-foreground">
						{index + 1}
					</span>
					<span className="min-w-0 flex-1">
						<span className="block truncate text-sm font-medium">
							{line.label.trim() || `Ligne ${index + 1}`}
						</span>
						{!isOpen ? (
							<span className="block truncate text-xs text-muted-foreground">
								{invoiceLineSummary(line)}
							</span>
						) : null}
					</span>
				</CollapsibleTrigger>

				<span className="shrink-0 text-sm font-semibold tabular-nums">
					{formatMoney(lineTotalCents)}
				</span>
				<Button
					type="button"
					variant="ghost"
					size="icon-sm"
					disabled={!canRemove}
					onClick={onRemove}
				>
					<Trash2 />
					<span className="sr-only">Supprimer la ligne</span>
				</Button>
			</div>

			<CollapsibleContent>
				<div className="space-y-4 border-t px-4 py-4">
					<Field label="Prestation" htmlFor={`label-${line.clientId}`}>
						<Input
							id={`label-${line.clientId}`}
							value={line.label}
							onChange={(event) => onChange({ label: event.target.value })}
							placeholder="Ce qui sera facturé"
						/>
					</Field>

					<div
						className={cn(
							'grid items-end gap-4',
							vatEnabled
								? 'sm:grid-cols-[1fr_1fr_1fr_auto]'
								: 'sm:grid-cols-[1fr_1fr_auto]',
						)}
					>
						<Field label="Quantité" htmlFor={`quantity-${line.clientId}`}>
							<Input
								id={`quantity-${line.clientId}`}
								inputMode="decimal"
								value={line.quantity}
								onChange={(event) => onChange({ quantity: event.target.value })}
							/>
						</Field>
						<Field label="Prix unitaire" htmlFor={`price-${line.clientId}`}>
							<Input
								id={`price-${line.clientId}`}
								inputMode="decimal"
								value={line.unitPrice}
								onChange={(event) =>
									onChange({ unitPrice: event.target.value })
								}
								placeholder="0,00"
							/>
						</Field>
						{vatEnabled ? (
							<Field label="TVA" htmlFor={`vat-${line.clientId}`}>
								<VatRateField
									id={`vat-${line.clientId}`}
									valueBp={line.vatRateBp}
									onChange={(vatRateBp) => onChange({ vatRateBp })}
								/>
							</Field>
						) : null}
						<div className="rounded-lg bg-muted px-4 py-2 text-right sm:min-w-32">
							<p className="text-xs text-muted-foreground">Total ligne</p>
							<p className="font-semibold tabular-nums">
								{formatMoney(lineTotalCents)}
							</p>
						</div>
					</div>
				</div>
			</CollapsibleContent>
		</Collapsible>
	)
}

const CUSTOM_VAT_RATE = 'custom'

function VatRateField({
	id,
	valueBp,
	onChange,
}: {
	id: string
	valueBp: string
	onChange: (valueBp: string) => void
}) {
	const isCommonRate =
		valueBp !== '' &&
		(COMMON_VAT_RATES_BP as readonly number[]).includes(Number(valueBp))
	const selectValue =
		valueBp === '' ? '' : isCommonRate ? valueBp : CUSTOM_VAT_RATE

	return (
		<div className="space-y-1.5">
			<Select
				value={selectValue}
				onValueChange={(next) => {
					if (next === CUSTOM_VAT_RATE) {
						onChange(valueBp || '0')
						return
					}
					onChange(next)
				}}
			>
				<SelectTrigger id={id} className="w-full">
					<SelectValue placeholder="Taux" />
				</SelectTrigger>
				<SelectContent>
					{COMMON_VAT_RATES_BP.map((rateBp) => (
						<SelectItem key={rateBp} value={String(rateBp)}>
							{formatVatRateBp(rateBp)}
						</SelectItem>
					))}
					<SelectItem value={CUSTOM_VAT_RATE}>Autre…</SelectItem>
				</SelectContent>
			</Select>
			{selectValue === CUSTOM_VAT_RATE ? (
				<Input
					inputMode="decimal"
					value={valueBp === '' ? '' : String(Number(valueBp) / 100)}
					onChange={(event) => {
						const percent = Number(event.target.value.replace(',', '.'))
						onChange(
							Number.isFinite(percent)
								? String(Math.round(percent * 100))
								: '0',
						)
					}}
					placeholder="Taux en %"
				/>
			) : null}
		</div>
	)
}

function Field({
	label,
	htmlFor,
	children,
}: {
	label: string
	htmlFor?: string
	children: React.ReactNode
}) {
	return (
		<div className="space-y-1.5">
			<Label htmlFor={htmlFor} className="text-xs text-muted-foreground">
				{label}
			</Label>
			{children}
		</div>
	)
}
