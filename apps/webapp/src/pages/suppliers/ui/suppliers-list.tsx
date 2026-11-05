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
import type { Supplier } from '#/hooks/use-suppliers'
import { hueOf, NameAvatar } from '#/pages/customers/ui/avatars'
import { Notice } from '#/pages/customers/ui/notice'
import { SearchBox } from '#/pages/customers/ui/search-box'
import { formatLegalId } from '#/pages/suppliers/lib/suppliers-search'
import { firstAddressLine } from '#/pages/suppliers/types'

const DASH = <span className="text-muted-foreground">—</span>

export interface SuppliersListProps {
	suppliers: Supplier[]
	totalSuppliers: number
	query: string
	onQueryChange: (query: string) => void
	isLoading: boolean
	error: string | null
	canManage: boolean
	notice: string | null
	onDismissNotice: () => void
	onCreate: () => void
	onEdit: (supplier: Supplier) => void
}

export function SuppliersList({
	suppliers,
	totalSuppliers,
	query,
	onQueryChange,
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
			<div className="mb-3 flex flex-wrap items-center gap-2">
				<SearchBox
					value={query}
					onChange={onQueryChange}
					label="Rechercher un fournisseur"
					placeholder="Nom, SIRET, e-mail, adresse"
				/>
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
			) : suppliers.length === 0 ? (
				<EmptyState
					title={
						searching
							? 'Aucun fournisseur ne correspond à cette recherche'
							: 'Aucun fournisseur pour le moment'
					}
					hint={
						searching
							? 'Essaie un autre nom ou un autre SIRET.'
							: 'Ajoute ton premier fournisseur pour le rattacher aux produits du catalogue.'
					}
				>
					{!searching && totalSuppliers === 0 && (
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
								<TableHead className="max-lg:hidden">Contact</TableHead>
								<TableHead className="pr-4 max-md:hidden">Adresse</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{suppliers.map((supplier) => {
								const legalId = supplier.registration_number ?? ''
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
															onClick={() => onEdit(supplier)}
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
											{supplier.email || supplier.phone ? (
												<>
													<span className="block max-w-44 truncate">
														{supplier.email || supplier.phone}
													</span>
													{supplier.email && supplier.phone && (
														<span className="block max-w-44 truncate text-[12px] text-muted-foreground">
															{supplier.phone}
														</span>
													)}
												</>
											) : (
												DASH
											)}
										</TableCell>
										<TableCell className="pr-4 max-md:hidden">
											{firstAddressLine(supplier.address) || DASH}
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
