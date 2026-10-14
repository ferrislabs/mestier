import { AlertCircle, Archive, Copy, Loader2, Pencil, Plus } from 'lucide-react'
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
import { PageHeader, PageShell } from '#/components/ui/surface'
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
	organizationName: string
	season: number
	rows: ContractRow[]
	includeArchived: boolean
	isLoading: boolean
	error: string | null
	/** Set after "Dupliquer": what was made and where to see it. */
	notice: { message: string; season: number } | null
	pendingContractId: string | null
	archiveTarget: ContractRow | null
	form: ContractFormDialogProps
	onSeasonChange: (season: number) => void
	onIncludeArchivedChange: (includeArchived: boolean) => void
	onCreate: () => void
	onEdit: (contract: MaintenanceContract) => void
	onDuplicate: (contract: MaintenanceContract) => void
	onAskArchive: (row: ContractRow | null) => void
	onConfirmArchive: () => void
	onDismissNotice: () => void
}

const euros = new Intl.NumberFormat('fr-FR', {
	style: 'currency',
	currency: 'EUR',
})

export function ContractsUi({
	organizationName,
	season,
	rows,
	includeArchived,
	isLoading,
	error,
	notice,
	pendingContractId,
	archiveTarget,
	form,
	onSeasonChange,
	onIncludeArchivedChange,
	onCreate,
	onEdit,
	onDuplicate,
	onAskArchive,
	onConfirmArchive,
	onDismissNotice,
}: ContractsUiProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
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

			<label className="flex w-fit items-center gap-2 text-sm">
				<input
					type="checkbox"
					checked={includeArchived}
					onChange={(event) => onIncludeArchivedChange(event.target.checked)}
				/>
				Afficher les contrats archivés
			</label>

			{notice ? (
				<output className="flex flex-wrap items-center gap-3 border border-emerald-600/40 bg-emerald-50 p-3 text-sm text-emerald-900 dark:bg-emerald-950 dark:text-emerald-100">
					<span>{notice.message}</span>
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
				</output>
			) : null}

			{error ? (
				<div
					role="alert"
					className="flex items-center gap-2 border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive"
				>
					<AlertCircle className="size-4" />
					{error}
				</div>
			) : null}

			{isLoading ? (
				<div className="flex items-center gap-2 text-sm text-muted-foreground">
					<Loader2 className="size-4 animate-spin" />
					Chargement des contrats
				</div>
			) : null}

			{!isLoading && !error && rows.length === 0 ? (
				<p className="border bg-card p-6 text-sm text-muted-foreground">
					Aucun contrat pour la saison {season}.
				</p>
			) : null}

			{rows.length > 0 ? (
				<div className="overflow-x-auto border bg-card">
					<table className="w-full text-sm">
						<thead>
							<tr className="border-b text-left text-muted-foreground">
								<th scope="col" className="px-3 py-2 font-medium">
									Contrat
								</th>
								<th scope="col" className="px-3 py-2 font-medium">
									Saison
								</th>
								<th scope="col" className="px-3 py-2 font-medium">
									Prestations
								</th>
								<th scope="col" className="px-3 py-2 text-right font-medium">
									Montant
								</th>
								<th scope="col" className="px-3 py-2 font-medium">
									<span className="sr-only">Actions</span>
								</th>
							</tr>
						</thead>
						<tbody>
							{rows.map((row) => (
								<ContractTableRow
									key={row.contract.id}
									row={row}
									isPending={pendingContractId === row.contract.id}
									onEdit={onEdit}
									onDuplicate={onDuplicate}
									onAskArchive={onAskArchive}
								/>
							))}
						</tbody>
					</table>
				</div>
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
		</PageShell>
	)
}

interface ContractTableRowProps {
	row: ContractRow
	isPending: boolean
	onEdit: (contract: MaintenanceContract) => void
	onDuplicate: (contract: MaintenanceContract) => void
	onAskArchive: (row: ContractRow) => void
}

function ContractTableRow({
	row,
	isPending,
	onEdit,
	onDuplicate,
	onAskArchive,
}: ContractTableRowProps) {
	const { contract } = row
	const archived = contract.archived_at != null
	const place = placeLabel(row.customerName, row.siteLabel)

	return (
		<tr className="border-b align-top last:border-b-0">
			<td className="px-3 py-3">
				<div className="font-medium">{place}</div>
				<div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
					{contract.reference ? <span>{contract.reference}</span> : null}
					{row.managerName ? <span>{row.managerName}</span> : null}
					{contract.status === 'DRAFT' ? (
						<Badge variant="outline">Brouillon</Badge>
					) : null}
					{archived ? <Badge variant="secondary">Archivé</Badge> : null}
				</div>
			</td>
			<td className="px-3 py-3 whitespace-nowrap">
				{contract.season.year}
				<div className="text-xs text-muted-foreground">
					{seasonLabel(contract.season)}
				</div>
			</td>
			<td className="px-3 py-3">
				<ul className="space-y-0.5">
					{lineSummaries(contract).map((summary, index) => (
						// biome-ignore lint/suspicious/noArrayIndexKey: summaries are display strings that may repeat
						<li key={index}>{summary}</li>
					))}
				</ul>
			</td>
			<td className="px-3 py-3 text-right whitespace-nowrap">
				{contract.amount_cents == null
					? '-'
					: euros.format(contract.amount_cents / 100)}
			</td>
			<td className="px-3 py-3">
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
			</td>
		</tr>
	)
}
