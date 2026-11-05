import {
	AlertCircle,
	Archive,
	CheckCircle2,
	Copy,
	Loader2,
	Pencil,
	Plus,
} from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { duplicateLabel } from '#/pages/maintenance/lib/contract-form'
import {
	lineSummaries,
	placeLabel,
} from '#/pages/maintenance/lib/contract-list'
import { seasonLabel } from '#/pages/maintenance/lib/grid-model'
import type { MaintenanceContract } from '#/pages/maintenance/types'
import {
	ContractFormDialog,
	type ContractFormDialogProps,
} from '#/pages/maintenance/ui/contract-form-dialog'
import { SeasonPicker } from '#/pages/maintenance/ui/season-picker'

export interface ContractRow {
	contract: MaintenanceContract
	customerName: string | null
	siteLabel: string | null
	managerName: string | null
}

export interface ContractsUiProps {
	season: number
	/** The contracts of the season that are not drafts to review. */
	rows: ContractRow[]
	/** Draft contracts, waiting for a person to read them and activate them. */
	draftRows: ContractRow[]
	includeArchived: boolean
	isLoading: boolean
	error: string | null
	/** Set after "Dupliquer": what was made and where to see it. */
	notice: { message: string; season: number } | null
	pendingContractId: string | null
	archiveTarget: ContractRow | null
	activateTarget: ContractRow | null
	form: ContractFormDialogProps
	onSeasonChange: (season: number) => void
	onIncludeArchivedChange: (includeArchived: boolean) => void
	onCreate: () => void
	onEdit: (contract: MaintenanceContract) => void
	onDuplicate: (contract: MaintenanceContract) => void
	onAskArchive: (row: ContractRow | null) => void
	onConfirmArchive: () => void
	onAskActivate: (row: ContractRow | null) => void
	onConfirmActivate: () => void
	onDismissNotice: () => void
}

const euros = new Intl.NumberFormat('fr-FR', {
	style: 'currency',
	currency: 'EUR',
})

export function ContractsUi({
	season,
	rows,
	draftRows,
	includeArchived,
	isLoading,
	error,
	notice,
	pendingContractId,
	archiveTarget,
	activateTarget,
	form,
	onSeasonChange,
	onIncludeArchivedChange,
	onCreate,
	onEdit,
	onDuplicate,
	onAskArchive,
	onConfirmArchive,
	onAskActivate,
	onConfirmActivate,
	onDismissNotice,
}: ContractsUiProps) {
	const nothingToShow = rows.length === 0 && draftRows.length === 0

	return (
		<PageShell>
			<PageHeader
				title="Contrats d'entretien"
				description="Un contrat couvre une saison. Dupliquez-le pour l'année suivante."
				actions={
					<div className="flex flex-wrap items-center gap-3">
						<SeasonPicker season={season} onChange={onSeasonChange} />
						<Button type="button" onClick={onCreate}>
							<Plus />
							Nouveau contrat
						</Button>
					</div>
				}
			/>

			<div className="flex w-fit items-center gap-2 text-[13px]">
				<Checkbox
					id="contracts-include-archived"
					checked={includeArchived}
					onCheckedChange={(checked) =>
						onIncludeArchivedChange(checked === true)
					}
				/>
				<label htmlFor="contracts-include-archived">
					Afficher les contrats archivés
				</label>
			</div>

			{notice ? (
				<SectionCard
					role="status"
					className="flex flex-wrap items-center gap-3 border-success/30 bg-success-soft p-4 text-[13px] text-success"
				>
					<span className="font-medium">{notice.message}</span>
					<Button
						type="button"
						variant="outline"
						size="sm"
						onClick={() => onSeasonChange(notice.season)}
					>
						Voir la saison {notice.season}
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="sm"
						onClick={onDismissNotice}
					>
						Fermer
					</Button>
				</SectionCard>
			) : null}

			{error ? (
				<SectionCard
					role="alert"
					className="flex items-center gap-3 border-destructive/30 bg-destructive-soft p-4 text-destructive"
				>
					<AlertCircle className="size-4 shrink-0" />
					<p className="text-[13px] font-medium">{error}</p>
				</SectionCard>
			) : null}

			{isLoading ? (
				<div className="flex items-center gap-2 text-[13px] text-muted-foreground">
					<Loader2 className="size-4 animate-spin" />
					Chargement des contrats
				</div>
			) : null}

			{!isLoading && !error && nothingToShow ? (
				<EmptyState
					title={`Aucun contrat pour la saison ${season}.`}
					hint="Créez un contrat ou dupliquez celui d'une saison précédente."
				/>
			) : null}

			{draftRows.length > 0 ? (
				<section
					aria-labelledby="draft-contracts-title"
					className="space-y-3 rounded-xl border border-warning/30 bg-warning-soft/40 p-4"
				>
					<div className="space-y-1">
						<h2 className="flex items-center gap-2 text-base font-semibold">
							<span id="draft-contracts-title">Brouillons à vérifier</span>
							<Badge variant="outline">{draftRows.length}</Badge>
						</h2>
						<p className="text-[13px] text-muted-foreground">
							Ces contrats n'ont aucune échéance tant qu'ils ne sont pas activés
							: ils sont absents du récap. Relisez-les, corrigez-les si besoin,
							puis activez-les.
						</p>
					</div>
					<ContractsTable
						rows={draftRows}
						mode="draft"
						pendingContractId={pendingContractId}
						onEdit={onEdit}
						onDuplicate={onDuplicate}
						onAskArchive={onAskArchive}
						onAskActivate={onAskActivate}
					/>
				</section>
			) : null}

			{rows.length > 0 ? (
				<section aria-labelledby="active-contracts-title" className="space-y-3">
					<h2 id="active-contracts-title" className="text-base font-semibold">
						Contrats actifs
					</h2>
					<ContractsTable
						rows={rows}
						mode="active"
						pendingContractId={pendingContractId}
						onEdit={onEdit}
						onDuplicate={onDuplicate}
						onAskArchive={onAskArchive}
						onAskActivate={onAskActivate}
					/>
				</section>
			) : null}

			{!isLoading && !error && rows.length === 0 && draftRows.length > 0 ? (
				<p className="text-[13px] text-muted-foreground">
					Aucun contrat actif pour la saison {season}.
				</p>
			) : null}

			<ContractFormDialog {...form} />

			<AlertDialog
				open={archiveTarget !== null}
				onOpenChange={(open) => !open && onAskArchive(null)}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Archiver ce contrat ?</AlertDialogTitle>
						<AlertDialogDescription>
							{archiveTarget
								? `${placeLabel(archiveTarget.customerName, archiveTarget.siteLabel)} disparaît de la grille et du récap. Ses interventions sont conservées.`
								: null}
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Annuler</AlertDialogCancel>
						<AlertDialogAction onClick={onConfirmArchive}>
							Archiver
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>

			<AlertDialog
				open={activateTarget !== null}
				onOpenChange={(open) => !open && onAskActivate(null)}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Activer ce contrat ?</AlertDialogTitle>
						<AlertDialogDescription>
							{activateTarget
								? `${placeLabel(activateTarget.customerName, activateTarget.siteLabel)} entre dans la grille et dans le récap : ses prestations auront des échéances.`
								: null}
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Annuler</AlertDialogCancel>
						<AlertDialogAction onClick={onConfirmActivate}>
							Activer
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</PageShell>
	)
}

interface ContractsTableProps {
	rows: ContractRow[]
	mode: 'active' | 'draft'
	pendingContractId: string | null
	onEdit: (contract: MaintenanceContract) => void
	onDuplicate: (contract: MaintenanceContract) => void
	onAskArchive: (row: ContractRow) => void
	onAskActivate: (row: ContractRow) => void
}

function ContractsTable({
	rows,
	mode,
	pendingContractId,
	...actions
}: ContractsTableProps) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead scope="col" className="pl-4">
							Contrat
						</TableHead>
						<TableHead scope="col">Saison</TableHead>
						<TableHead scope="col">Prestations</TableHead>
						<TableHead scope="col" className="text-right">
							Montant
						</TableHead>
						<TableHead scope="col" className="pr-4">
							<span className="sr-only">Actions</span>
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{rows.map((row) => (
						<ContractTableRow
							key={row.contract.id}
							row={row}
							mode={mode}
							isPending={pendingContractId === row.contract.id}
							{...actions}
						/>
					))}
				</TableBody>
			</Table>
		</div>
	)
}

interface ContractTableRowProps {
	row: ContractRow
	mode: 'active' | 'draft'
	isPending: boolean
	onEdit: (contract: MaintenanceContract) => void
	onDuplicate: (contract: MaintenanceContract) => void
	onAskArchive: (row: ContractRow) => void
	onAskActivate: (row: ContractRow) => void
}

function ContractTableRow({
	row,
	mode,
	isPending,
	onEdit,
	onDuplicate,
	onAskArchive,
	onAskActivate,
}: ContractTableRowProps) {
	const { contract } = row
	const archived = contract.archived_at != null
	const place = placeLabel(row.customerName, row.siteLabel)

	return (
		<TableRow className="align-top">
			<TableCell className="py-3 pl-4 whitespace-normal">
				<div className="font-medium">{place}</div>
				<div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
					{contract.reference ? <span>{contract.reference}</span> : null}
					{row.managerName ? <span>{row.managerName}</span> : null}
					{contract.status === 'DRAFT' ? (
						<Badge variant="outline">Brouillon</Badge>
					) : null}
					{archived ? <Badge variant="secondary">Archivé</Badge> : null}
				</div>
			</TableCell>
			<TableCell className="py-3 tabular">
				{contract.season.year}
				<div className="text-xs text-muted-foreground">
					{seasonLabel(contract.season)}
				</div>
			</TableCell>
			<TableCell className="py-3 whitespace-normal">
				<ul className="space-y-0.5">
					{lineSummaries(contract).map((summary, index) => (
						// biome-ignore lint/suspicious/noArrayIndexKey: summaries are display strings that may repeat
						<li key={index}>{summary}</li>
					))}
				</ul>
			</TableCell>
			<TableCell className="py-3 text-right tabular">
				{contract.amount_cents == null
					? '-'
					: euros.format(contract.amount_cents / 100)}
			</TableCell>
			<TableCell className="py-3 pr-4">
				<div className="flex justify-end gap-1">
					<Button
						type="button"
						variant="ghost"
						size="sm"
						disabled={isPending}
						aria-label={`Modifier ${place}`}
						onClick={() => onEdit(contract)}
					>
						<Pencil />
						Modifier
					</Button>
					{mode === 'draft' ? (
						<Button
							type="button"
							size="sm"
							disabled={isPending}
							aria-label={`Activer ${place}`}
							onClick={() => onAskActivate(row)}
						>
							{isPending ? (
								<Loader2 className="animate-spin" />
							) : (
								<CheckCircle2 />
							)}
							Activer
						</Button>
					) : (
						<Button
							type="button"
							variant="ghost"
							size="sm"
							disabled={isPending}
							aria-label={`${duplicateLabel(contract)} : ${place}`}
							onClick={() => onDuplicate(contract)}
						>
							<Copy />
							{duplicateLabel(contract)}
						</Button>
					)}
					{archived ? null : (
						<Button
							type="button"
							variant="ghost"
							size="sm"
							disabled={isPending}
							aria-label={`Archiver ${place}`}
							onClick={() => onAskArchive(row)}
						>
							<Archive />
							Archiver
						</Button>
					)}
				</div>
			</TableCell>
		</TableRow>
	)
}
