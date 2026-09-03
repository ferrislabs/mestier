import { MapPin } from 'lucide-react'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { CustomerContext } from '#/hooks/use-customers'
import { billingAddressLines } from '../types'

interface BillingAddressFieldProps {
	id?: string
	value: string
	addresses: CustomerContext[]
	hasCustomer: boolean
	isLoading?: boolean
	onChange: (customerContextId: string) => void
}

/**
 * Picks the address a quote is billed to.
 *
 * The option list shows the label, because that is what an artisan named the
 * place; the selected address is then written out in full underneath, because
 * that is what ends up on the document. Showing only the label was the reason
 * nobody could tell two sites of the same customer apart.
 */
export function BillingAddressField({
	id,
	value,
	addresses,
	hasCustomer,
	isLoading,
	onChange,
}: BillingAddressFieldProps) {
	const selected = addresses.find((address) => address.id === value)
	const lines = selected ? billingAddressLines(selected) : []

	return (
		<div className="space-y-1.5">
			<Select
				value={value}
				onValueChange={onChange}
				disabled={!hasCustomer || isLoading}
			>
				<SelectTrigger id={id} className="w-full">
					<SelectValue
						placeholder={
							hasCustomer
								? 'Sélectionner une adresse'
								: 'Choisir un client d’abord'
						}
					/>
				</SelectTrigger>
				<SelectContent>
					{addresses.map((address) => (
						<SelectItem key={address.id} value={address.id}>
							{address.label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>

			{selected ? (
				<p className="flex items-start gap-1.5 text-xs text-muted-foreground">
					<MapPin className="mt-0.5 size-3 shrink-0" />
					{lines.length > 0 ? (
						<span>
							{lines.map((part) => (
								<span key={part} className="block">
									{part}
								</span>
							))}
						</span>
					) : (
						<span className="text-amber-600 dark:text-amber-500">
							Aucune adresse renseignée pour ce lieu. Le devis partira sans
							adresse de facturation.
						</span>
					)}
				</p>
			) : null}
		</div>
	)
}
