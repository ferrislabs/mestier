import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { SearchBox } from '#/pages/customers/ui/search-box'
import {
	hasStripeFilters,
	type StripeSearch,
	type StripeSearchPatch,
} from '#/pages/stripe/lib/stripe-model'
import {
	STRIPE_STATUS_LABELS,
	STRIPE_TYPE_LABELS,
	type StripeEventStatus,
	type StripeEventType,
} from '#/pages/stripe/types'

const ALL = '__all'

const STATUS_CHOICES: readonly StripeEventStatus[] = [
	'to_review',
	'processed',
	'ignored',
]

const TYPE_CHOICES: readonly StripeEventType[] = [
	'payment_intent.succeeded',
	'invoice.paid',
	'charge.refunded',
]

export function StripeToolbar({
	search,
	onChange,
}: {
	search: StripeSearch
	onChange: (patch: StripeSearchPatch) => void
}) {
	return (
		<div className="mb-3 flex flex-wrap items-center gap-2">
			<SearchBox
				value={search.q ?? ''}
				onChange={(q) => onChange({ q })}
				label="Rechercher dans l’historique Stripe"
				placeholder="Client, événement, facture"
			/>
			<Select
				value={search.statut ?? ALL}
				onValueChange={(value) =>
					onChange({
						statut: value === ALL ? null : (value as StripeEventStatus),
					})
				}
			>
				<SelectTrigger size="sm" className="w-44 bg-card" aria-label="Statut">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={ALL}>Tous les statuts</SelectItem>
					{STATUS_CHOICES.map((status) => (
						<SelectItem key={status} value={status}>
							{STRIPE_STATUS_LABELS[status]}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			<Select
				value={search.type ?? ALL}
				onValueChange={(value) =>
					onChange({ type: value === ALL ? null : (value as StripeEventType) })
				}
			>
				<SelectTrigger
					size="sm"
					className="w-52 bg-card"
					aria-label="Type d’événement"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={ALL}>Tous les types</SelectItem>
					{TYPE_CHOICES.map((type) => (
						<SelectItem key={type} value={type}>
							{STRIPE_TYPE_LABELS[type]}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			{hasStripeFilters(search) && (
				<Button
					type="button"
					variant="ghost"
					size="sm"
					onClick={() => onChange({ statut: null, type: null, q: null })}
				>
					Réinitialiser
				</Button>
			)}
		</div>
	)
}
