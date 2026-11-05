import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { formatMoney } from '#/pages/customers/lib/format'
import type { OpenInvoice } from '#/pages/treasury/types'

const NONE = '__none'

export function invoiceOptionLabel(option: OpenInvoice): string {
	return `${option.label} · ${option.customerName} · reste ${formatMoney(option.balance)}`
}

export function OpenInvoiceSelect({
	id,
	options,
	value,
	onChange,
	amount,
	noneLabel,
	placeholder = 'Choisir une facture',
	disabled,
	invalid,
}: {
	id?: string
	options: readonly OpenInvoice[]
	value: string | null
	onChange: (invoiceId: string | null) => void
	amount?: number
	noneLabel?: string
	placeholder?: string
	disabled?: boolean
	invalid?: boolean
}) {
	return (
		<Select
			value={value ?? (noneLabel ? NONE : '')}
			onValueChange={(next) => onChange(next === NONE ? null : next)}
			disabled={disabled}
		>
			<SelectTrigger
				id={id}
				className="w-full"
				aria-invalid={invalid || undefined}
			>
				<SelectValue placeholder={placeholder} />
			</SelectTrigger>
			<SelectContent>
				{noneLabel && <SelectItem value={NONE}>{noneLabel}</SelectItem>}
				{options.map((option) => (
					<SelectItem
						key={option.id}
						value={option.id}
						disabled={amount !== undefined && amount > option.balance}
					>
						{invoiceOptionLabel(option)}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	)
}
