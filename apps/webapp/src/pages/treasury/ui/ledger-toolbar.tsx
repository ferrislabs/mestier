import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { formatMonth } from '#/pages/customers/lib/format'
import { SearchBox } from '#/pages/customers/ui/search-box'
import {
	hasLedgerFilters,
	type LedgerSearch,
	type LedgerSearchPatch,
	ledgerStatusOptions,
} from '#/pages/treasury/lib/ledger-model'
import {
	type ExpenseCategory,
	LEDGER_STATUS_LABELS,
} from '#/pages/treasury/types'

const ALL = '__all'

function capitalized(text: string): string {
	return text.charAt(0).toUpperCase() + text.slice(1)
}

export function LedgerToolbar({
	search,
	months,
	categories,
	onChange,
}: {
	search: LedgerSearch
	months: readonly string[]
	categories: readonly ExpenseCategory[]
	onChange: (patch: LedgerSearchPatch) => void
}) {
	const monthChoices =
		search.mois && !months.includes(search.mois)
			? [search.mois, ...months]
			: months
	const onlyReceipts = search.type === 'recettes'
	return (
		<div className="mb-3 flex flex-wrap items-center gap-2">
			<SearchBox
				value={search.q ?? ''}
				onChange={(q) => onChange({ q })}
				label="Rechercher dans le journal"
				placeholder="Libellé, tiers, référence"
			/>
			<Select
				value={search.mois ?? ALL}
				onValueChange={(value) =>
					onChange({ mois: value === ALL ? null : value })
				}
			>
				<SelectTrigger size="sm" className="w-44 bg-card" aria-label="Mois">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={ALL}>Tous les mois</SelectItem>
					{monthChoices.map((month) => (
						<SelectItem key={month} value={month}>
							{capitalized(formatMonth(month))}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			<Select
				value={search.categorie ?? ALL}
				onValueChange={(value) =>
					onChange({ categorie: value === ALL ? null : value })
				}
				disabled={onlyReceipts}
			>
				<SelectTrigger
					size="sm"
					className="w-52 bg-card"
					aria-label="Catégorie de dépense"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={ALL}>Toutes les catégories</SelectItem>
					{categories.map((category) => (
						<SelectItem key={category.id} value={category.id}>
							{category.name}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			<Select
				value={search.statut ?? ALL}
				onValueChange={(value) =>
					onChange({
						statut: value === ALL ? null : (value as LedgerSearch['statut']),
					})
				}
			>
				<SelectTrigger size="sm" className="w-44 bg-card" aria-label="Statut">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={ALL}>Tous les statuts</SelectItem>
					{ledgerStatusOptions(search.type).map((status) => (
						<SelectItem key={status} value={status}>
							{LEDGER_STATUS_LABELS[status]}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			{hasLedgerFilters(search) && (
				<Button
					type="button"
					variant="ghost"
					size="sm"
					onClick={() =>
						onChange({ mois: null, q: null, categorie: null, statut: null })
					}
				>
					Réinitialiser
				</Button>
			)}
		</div>
	)
}
