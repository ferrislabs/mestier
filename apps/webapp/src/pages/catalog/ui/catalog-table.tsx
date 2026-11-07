import {
	Archive,
	ArchiveRestore,
	Copy,
	Ellipsis,
	Pencil,
	Trash2,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { formatUnitLong } from '#/lib/units'
import { cn } from '#/lib/utils'
import {
	CATALOG_KIND_LABELS,
	type CatalogRow,
	type CatalogUsage,
	FREQUENCY_SUFFIX,
	formatPercent,
	type MarginTone,
	marginOf,
	marginTone,
	usageLabel,
} from '#/pages/catalog/catalog-model'
import type { CatalogExtras } from '#/pages/catalog/feature/mocks/catalog-extras'
import { formatVatRate } from '#/pages/crm-settings/types'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { Money } from '#/pages/customers/ui/money'

const MARGIN_CLASSES: Record<MarginTone, string> = {
	ok: 'text-state-progress',
	warn: 'text-state-today',
	alert: 'text-state-alert',
}

function unitLabel(row: CatalogRow): string {
	return row.kind === 'product' && row.unit === 'HOUR'
		? 'unité'
		: formatUnitLong(row.unit)
}

function UsageCell({ usage }: { usage: CatalogUsage }) {
	const { main, detail } = usageLabel(usage)
	if (detail === null)
		return <span className="text-muted-foreground">{main}</span>
	return (
		<>
			<span className="tabular block">{main}</span>
			<span className="tabular block text-[12px] text-muted-foreground">
				{detail}
			</span>
		</>
	)
}

function MarginCell({
	row,
	extras,
}: {
	row: CatalogRow
	extras: CatalogExtras
}) {
	const margin = marginOf(row.unitPriceCents, extras.cost)
	if (!margin) return <span className="text-muted-foreground">—</span>
	return (
		<span
			className={cn(
				'tabular block',
				MARGIN_CLASSES[marginTone(margin.percent)],
			)}
		>
			<Money cents={margin.amount} className="block" />
			<span className="text-[12px]">{formatPercent(margin.percent)}</span>
		</span>
	)
}

export function CatalogTable({
	rows,
	extrasOf,
	usageOf,
	canManage,
	onEdit,
	onDuplicate,
	onToggleArchive,
	onDelete,
}: {
	rows: readonly CatalogRow[]
	extrasOf: (itemId: string) => CatalogExtras
	usageOf: (itemId: string) => CatalogUsage
	canManage: boolean
	onEdit: (row: CatalogRow) => void
	onDuplicate: (row: CatalogRow) => void
	onToggleArchive: (row: CatalogRow) => void
	onDelete: (row: CatalogRow) => void
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Référence</TableHead>
						<TableHead>Désignation</TableHead>
						<TableHead className="max-xl:hidden">Catégorie</TableHead>
						<TableHead className="max-lg:hidden">Unité</TableHead>
						<TableHead className="text-right">Prix HT</TableHead>
						<TableHead className="text-right max-lg:hidden">TVA</TableHead>
						<TableHead className="text-right max-xl:hidden">Coût</TableHead>
						<TableHead className="text-right">Marge</TableHead>
						<TableHead className="max-xl:hidden">Utilisation</TableHead>
						<TableHead>État</TableHead>
						<TableHead className="w-12 pr-4">
							<span className="sr-only">Actions</span>
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{rows.map((row) => {
						const extras = extrasOf(row.id)
						return (
							<TableRow
								key={row.id}
								className={cn(extras.archived && 'opacity-60')}
							>
								<TableCell className="py-2.5 pl-4 font-mono text-[12px] text-muted-foreground">
									{row.reference || '—'}
								</TableCell>
								<TableCell className="py-2.5">
									<div className="max-w-64 xl:max-w-80">
										{canManage ? (
											<button
												type="button"
												onClick={() => onEdit(row)}
												className="block max-w-full truncate rounded-sm text-left font-medium outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
											>
												{row.name}
											</button>
										) : (
											<span className="block max-w-full truncate font-medium">
												{row.name}
											</span>
										)}
										<span className="block truncate text-[12px] text-muted-foreground">
											<span className="font-medium">
												{CATALOG_KIND_LABELS[row.kind]}
											</span>
											{row.description && ` · ${row.description}`}
										</span>
									</div>
								</TableCell>
								<TableCell className="text-muted-foreground max-xl:hidden">
									{extras.category || '—'}
								</TableCell>
								<TableCell className="text-muted-foreground max-lg:hidden">
									{unitLabel(row)}
								</TableCell>
								<TableCell className="text-right">
									<Money cents={row.unitPriceCents} />
									{extras.frequency !== 'one_shot' && (
										<span className="block text-muted-foreground text-xs">
											{FREQUENCY_SUFFIX[extras.frequency]}
										</span>
									)}
								</TableCell>
								<TableCell className="tabular text-right max-lg:hidden">
									{row.vatRate === null ? (
										<span className="text-muted-foreground">—</span>
									) : (
										formatVatRate(row.vatRate)
									)}
								</TableCell>
								<TableCell className="text-right max-xl:hidden">
									{extras.cost === null ? (
										<span className="text-muted-foreground">—</span>
									) : (
										<Money cents={extras.cost} />
									)}
								</TableCell>
								<TableCell className="text-right">
									<MarginCell row={row} extras={extras} />
								</TableCell>
								<TableCell className="max-xl:hidden">
									<UsageCell usage={usageOf(row.id)} />
								</TableCell>
								<TableCell>
									<StatusBadge tone={extras.archived ? 'neutral' : 'ok'}>
										{extras.archived ? 'Archivé' : 'Actif'}
									</StatusBadge>
								</TableCell>
								<TableCell className="pr-4 text-right">
									{canManage && (
										<DropdownMenu>
											<DropdownMenuTrigger asChild>
												<Button
													variant="ghost"
													size="icon-sm"
													aria-label={`Actions pour ${row.name}`}
												>
													<Ellipsis aria-hidden />
												</Button>
											</DropdownMenuTrigger>
											<DropdownMenuContent align="end">
												<DropdownMenuItem onSelect={() => onEdit(row)}>
													<Pencil aria-hidden /> Modifier
												</DropdownMenuItem>
												<DropdownMenuItem onSelect={() => onDuplicate(row)}>
													<Copy aria-hidden /> Dupliquer
												</DropdownMenuItem>
												<DropdownMenuSeparator />
												<DropdownMenuItem onSelect={() => onToggleArchive(row)}>
													{extras.archived ? (
														<>
															<ArchiveRestore aria-hidden /> Restaurer
														</>
													) : (
														<>
															<Archive aria-hidden /> Archiver
														</>
													)}
												</DropdownMenuItem>
												<DropdownMenuItem
													variant="destructive"
													onSelect={() => onDelete(row)}
												>
													<Trash2 aria-hidden /> Supprimer
												</DropdownMenuItem>
											</DropdownMenuContent>
										</DropdownMenu>
									)}
								</TableCell>
							</TableRow>
						)
					})}
				</TableBody>
			</Table>
		</div>
	)
}
