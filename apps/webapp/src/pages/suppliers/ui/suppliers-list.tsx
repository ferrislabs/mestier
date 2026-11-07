import { Plus } from 'lucide-react'
import { DeveloperModeOnly } from '#/components/developer-mode-only'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { cn } from '#/lib/utils'
import { hueOf, NameAvatar } from '#/pages/customers/ui/avatars'
import { Money } from '#/pages/customers/ui/money'
import { Notice } from '#/pages/customers/ui/notice'
import { SearchBox } from '#/pages/customers/ui/search-box'
import { StatTile } from '#/pages/customers/ui/stat-tile'
import {
	formatLegalId,
	type SupplierRow,
} from '#/pages/suppliers/lib/supplier-model'
import { CategoryChip } from '#/pages/treasury/ui/category-chip'

const DASH = <span className="text-muted-foreground">—</span>

export interface SuppliersListProps {
	rows: readonly SupplierRow[]
	totals: { spend12m: number; toPay: number; overdue: boolean }
	totalSuppliers: number
	query: string
	onQueryChange: (query: string) => void
	archived: boolean
	onArchivedChange: (archived: boolean) => void
	isLoading: boolean
	error: string | null
	canManage: boolean
	notice: string | null
	onDismissNotice: () => void
	onCreate: () => void
	onEdit: (row: SupplierRow) => void
}

export function SuppliersList({
	rows,
	totals,
	totalSuppliers,
	query,
	onQueryChange,
	archived,
	onArchivedChange,
	isLoading,
	error,
	canManage,
	notice,
	onDismissNotice,
	onCreate,
	onEdit,
}: SuppliersListProps) {
	const searching = query.trim() !== ''
	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6 flex-row flex-wrap items-end justify-between gap-4"
				title="Fournisseurs"
				description="Ceux qui te facturent : matériaux, sous-traitance, assurances, loyers."
				actions={
					<RequirePermission permission="MANAGE_REFERENCE">
						<Button onClick={onCreate}>
							<Plus aria-hidden />
							Nouveau fournisseur
						</Button>
					</RequirePermission>
				}
			/>
			<Notice message={notice} onDismiss={onDismissNotice} />
			<div className="mb-4 grid gap-3 sm:grid-cols-2">
				<StatTile
					label="Dépenses 12 mois"
					value={<Money cents={totals.spend12m} />}
					hint="Montants TTC"
				/>
				<StatTile
					label="À payer"
					value={<Money cents={totals.toPay} />}
					tone={totals.overdue ? 'alert' : 'default'}
					hint={
						totals.overdue
							? 'Dont des échéances dépassées'
							: 'Dépenses à régler et en retard'
					}
				/>
			</div>
			<div className="mb-3 flex flex-wrap items-center gap-2">
				<SearchBox
					value={query}
					onChange={onQueryChange}
					label="Rechercher un fournisseur"
					placeholder="Nom, SIRET, contact, catégorie"
				/>
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={archived ? 'archived' : 'active'}
					onValueChange={(value) => {
						if (value) onArchivedChange(value === 'archived')
					}}
					aria-label="Filtrer par statut"
				>
					<ToggleGroupItem value="active">Actifs</ToggleGroupItem>
					<ToggleGroupItem value="archived">Archivés</ToggleGroupItem>
				</ToggleGroup>
			</div>
			{error ? (
				<div className="mb-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-[13px] font-medium text-state-alert">
					{error}
				</div>
			) : null}
			{isLoading ? (
				<div className="rounded-lg border bg-card px-6 py-12 text-center text-[13px] text-muted-foreground">
					Chargement des fournisseurs…
				</div>
			) : rows.length === 0 ? (
				<EmptyState
					title={
						searching
							? 'Aucun fournisseur ne correspond à cette recherche'
							: archived
								? 'Aucun fournisseur archivé'
								: 'Aucun fournisseur pour le moment'
					}
					hint={
						searching
							? 'Essaie un autre nom ou un autre SIRET.'
							: archived
								? 'Les fournisseurs que tu archives apparaissent ici.'
								: 'Ajoute ton premier fournisseur pour le rattacher aux produits du catalogue.'
					}
				>
					{!searching && !archived && totalSuppliers === 0 && (
						<RequirePermission permission="MANAGE_REFERENCE">
							<Button onClick={onCreate}>
								<Plus aria-hidden />
								Nouveau fournisseur
							</Button>
						</RequirePermission>
					)}
				</EmptyState>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Fournisseur</TableHead>
								<TableHead className="max-2xl:hidden">SIRET</TableHead>
								<TableHead className="max-lg:hidden">
									Contact principal
								</TableHead>
								<TableHead className="max-md:hidden">
									Catégorie par défaut
								</TableHead>
								<TableHead className="max-xl:hidden">Délai</TableHead>
								<TableHead className="text-right">Dépenses 12 mois</TableHead>
								<TableHead className="pr-4 text-right">À payer</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{rows.map((row) => {
								const supplier = row.supplier
								const legalId = supplier.registration_number ?? ''
								const terms = row.extras.paymentTermsDays
								return (
									<TableRow
										key={supplier.id}
										className="relative cursor-pointer focus-within:bg-muted/50"
									>
										<TableCell className="py-2.5 pl-4">
											<div className="flex items-center gap-3">
												<NameAvatar
													name={supplier.name}
													hue={hueOf(supplier.id)}
													size="lg"
												/>
												<div className="min-w-0">
													{canManage ? (
														<button
															type="button"
															onClick={() => onEdit(row)}
															className="block max-w-full truncate rounded-sm text-left font-medium outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
														>
															{supplier.name}
															<span className="sr-only">, ouvrir la fiche</span>
														</button>
													) : (
														<span className="block max-w-full truncate font-medium">
															{supplier.name}
														</span>
													)}
													{legalId !== '' && (
														<span className="tabular block text-[12px] text-muted-foreground 2xl:hidden">
															{formatLegalId(legalId)}
														</span>
													)}
													<DeveloperModeOnly>
														<span className="block truncate font-mono text-[12px] text-muted-foreground">
															{supplier.id}
														</span>
													</DeveloperModeOnly>
												</div>
											</div>
										</TableCell>
										<TableCell className="tabular whitespace-nowrap max-2xl:hidden">
											{legalId !== '' ? formatLegalId(legalId) : DASH}
										</TableCell>
										<TableCell className="max-lg:hidden">
											{row.contact !== '' || row.contactEmail !== '' ? (
												<>
													<span className="block max-w-44 truncate">
														{row.contact || row.contactEmail}
													</span>
													{row.contact !== '' && row.contactEmail !== '' && (
														<span className="block max-w-44 truncate text-[12px] text-muted-foreground">
															{row.contactEmail}
														</span>
													)}
												</>
											) : (
												DASH
											)}
										</TableCell>
										<TableCell className="max-md:hidden">
											<CategoryChip category={row.category ?? undefined} />
										</TableCell>
										<TableCell className="max-xl:hidden">
											{terms === null ? DASH : `${terms} jours`}
										</TableCell>
										<TableCell className="text-right">
											{row.spend12m > 0 ? <Money cents={row.spend12m} /> : DASH}
										</TableCell>
										<TableCell
											className={cn(
												'pr-4 text-right font-medium',
												row.overdue && 'text-state-alert',
											)}
										>
											{row.toPay > 0 ? (
												<>
													<Money cents={row.toPay} />
													{row.overdue && (
														<span className="block font-normal text-[12px]">
															dont des échéances dépassées
														</span>
													)}
												</>
											) : (
												DASH
											)}
										</TableCell>
									</TableRow>
								)
							})}
						</TableBody>
					</Table>
				</div>
			)}
		</PageShell>
	)
}
