import { Link } from '@tanstack/react-router'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { buildOrgPath } from '#/modules/org-path'
import { formatDate } from '#/pages/customers/lib/format'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { Money } from '#/pages/customers/ui/money'
import {
	type ExpenseCategory,
	type LedgerEntry,
	PAYMENT_METHOD_LABELS,
} from '#/pages/treasury/types'
import { CategoryChip } from '#/pages/treasury/ui/category-chip'
import { LedgerStatusBadge } from '#/pages/treasury/ui/ledger-badges'

const DASH = <span className="text-muted-foreground">—</span>

function PartyCell({
	entry,
	organizationSlug,
}: {
	entry: LedgerEntry
	organizationSlug: string
}) {
	if (entry.party === '') return DASH
	if (entry.kind === 'receipt' && entry.customerId !== null)
		return (
			<Link
				to={buildOrgPath(organizationSlug, '/crm/clients/$customerId')}
				params={{ customerId: entry.customerId }}
				className="relative rounded-sm outline-none transition-colors hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring"
			>
				{entry.party}
			</Link>
		)
	return <span>{entry.party}</span>
}

export function LedgerTable({
	entries,
	categories,
	organizationSlug,
	onOpen,
}: {
	entries: readonly LedgerEntry[]
	categories: readonly ExpenseCategory[]
	organizationSlug: string
	onOpen: (key: string) => void
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Date</TableHead>
						<TableHead>Libellé</TableHead>
						<TableHead className="max-md:hidden">Tiers</TableHead>
						<TableHead className="max-lg:hidden">
							Catégorie ou moyen de paiement
						</TableHead>
						<TableHead>Statut</TableHead>
						<TableHead className="pr-4 text-right">Montant</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{entries.map((entry) => (
						<TableRow
							key={entry.key}
							className="relative cursor-pointer focus-within:bg-muted/50"
						>
							<TableCell className="py-2.5 pl-4 whitespace-nowrap">
								<button
									type="button"
									onClick={() => onOpen(entry.key)}
									className="rounded-sm text-left outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
								>
									{formatDate(entry.date)}
									<span className="sr-only">
										, ouvrir le détail de {entry.label}
									</span>
								</button>
							</TableCell>
							<TableCell className="min-w-48 whitespace-normal">
								<span className="flex flex-wrap items-center gap-x-2 gap-y-1">
									<span className="font-medium">{entry.label}</span>
									{entry.origin === 'stripe' && (
										<StatusBadge tone="info">Stripe</StatusBadge>
									)}
								</span>
								{entry.reference !== '' && (
									<span className="block text-[12px] text-muted-foreground">
										{entry.reference}
									</span>
								)}
							</TableCell>
							<TableCell className="max-md:hidden">
								<PartyCell entry={entry} organizationSlug={organizationSlug} />
							</TableCell>
							<TableCell className="max-lg:hidden">
								{entry.kind === 'expense' ? (
									<CategoryChip
										category={categories.find(
											(category) => category.id === entry.categoryId,
										)}
									/>
								) : entry.method !== null ? (
									PAYMENT_METHOD_LABELS[entry.method]
								) : (
									DASH
								)}
							</TableCell>
							<TableCell>
								<LedgerStatusBadge status={entry.status} />
							</TableCell>
							<TableCell className="pr-4 text-right font-medium">
								<Money cents={entry.amount} signed />
							</TableCell>
						</TableRow>
					))}
				</TableBody>
			</Table>
		</div>
	)
}
