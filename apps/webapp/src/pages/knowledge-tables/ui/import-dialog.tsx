import { useId } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	CSV_ACCEPT,
	importSummary,
	invalidReasonLabel,
	moreInvalidCells,
} from '#/pages/knowledge-tables/lib/csv-transfer'
import type { TableImportReport } from '#/pages/knowledge-tables/lib/types'

export type ImportMode = 'table' | 'rows'

interface ImportDialogProps {
	open: boolean
	mode: ImportMode
	fileName: string | null
	name: string
	report: TableImportReport | null
	error: string | null
	isImporting: boolean
	onFileChange: (file: File | null) => void
	onNameChange: (name: string) => void
	onSubmit: () => void
	onFinish: () => void
	onOpenChange: (open: boolean) => void
}

const DESCRIPTIONS: Record<ImportMode, string> = {
	table:
		'Chaque en-tête devient une colonne, la première devient la colonne principale. Le type est deviné : nombre, date, case à cocher ou texte. Séparateur « ; » ou « , », UTF-8 ou export Excel, 5 Mo et 10 000 lignes au plus.',
	rows: 'Les en-têtes sont reconnus par le nom des colonnes, sans tenir compte des majuscules ; les autres sont ignorés. Une cellule invalide est laissée vide et signalée. Les lignes importées ne déclenchent aucun workflow.',
}

function ImportReportView({ report }: { report: TableImportReport }) {
	const more = moreInvalidCells(report)
	return (
		<div className="grid gap-3 text-sm">
			<output className="block">{importSummary(report)}</output>
			{report.ignored_columns.length > 0 ? (
				<div className="grid gap-1">
					<p className="font-medium">Colonnes ignorées</p>
					<p className="text-muted-foreground">
						{report.ignored_columns.join(', ')}
					</p>
				</div>
			) : null}
			{report.invalid_cells.length > 0 ? (
				<div className="grid gap-1">
					<p className="font-medium">Cellules laissées vides</p>
					<div className="max-h-64 overflow-auto rounded-md border">
						<table className="w-full text-left text-[13px]">
							<thead className="sticky top-0 bg-muted">
								<tr>
									<th className="px-2 py-1 font-medium">Ligne</th>
									<th className="px-2 py-1 font-medium">Colonne</th>
									<th className="px-2 py-1 font-medium">Valeur</th>
									<th className="px-2 py-1 font-medium">Raison</th>
								</tr>
							</thead>
							<tbody>
								{report.invalid_cells.map((cell) => (
									<tr
										key={`${cell.row}-${cell.column}`}
										className="border-t align-top"
									>
										<td className="px-2 py-1 tabular-nums">{cell.row}</td>
										<td className="px-2 py-1">{cell.column}</td>
										<td className="break-all px-2 py-1 text-muted-foreground">
											{cell.value}
										</td>
										<td className="px-2 py-1">
											{invalidReasonLabel(cell.reason)}
										</td>
									</tr>
								))}
							</tbody>
						</table>
					</div>
					{more > 0 ? (
						<p className="text-muted-foreground text-xs">
							{more === 1
								? 'Et 1 autre cellule.'
								: `Et ${more} autres cellules.`}
						</p>
					) : null}
				</div>
			) : null}
		</div>
	)
}

export function ImportDialog({
	open,
	mode,
	fileName,
	name,
	report,
	error,
	isImporting,
	onFileChange,
	onNameChange,
	onSubmit,
	onFinish,
	onOpenChange,
}: ImportDialogProps) {
	const baseId = useId()
	const title =
		report !== null
			? 'Import terminé'
			: mode === 'table'
				? 'Créer une table depuis un CSV'
				: 'Importer des lignes depuis un CSV'
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-2xl">
				<form
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						if (report !== null) onFinish()
						else onSubmit()
					}}
				>
					<DialogHeader>
						<DialogTitle>{title}</DialogTitle>
						{report === null ? (
							<DialogDescription>{DESCRIPTIONS[mode]}</DialogDescription>
						) : (
							<DialogDescription className="sr-only">
								Compte rendu de l’import
							</DialogDescription>
						)}
					</DialogHeader>
					{report !== null ? (
						<ImportReportView report={report} />
					) : (
						<>
							<div className="grid gap-1.5">
								<Label htmlFor={`${baseId}-file`}>Fichier CSV</Label>
								<Input
									id={`${baseId}-file`}
									type="file"
									accept={CSV_ACCEPT}
									onChange={(event) =>
										onFileChange(event.target.files?.[0] ?? null)
									}
								/>
								{fileName ? (
									<p className="truncate text-muted-foreground text-xs">
										{fileName}
									</p>
								) : null}
							</div>
							{mode === 'table' ? (
								<div className="grid gap-1.5">
									<Label htmlFor={`${baseId}-name`}>Nom de la table</Label>
									<Input
										id={`${baseId}-name`}
										value={name}
										maxLength={100}
										autoComplete="off"
										placeholder="Inventaire"
										onChange={(event) => onNameChange(event.target.value)}
									/>
								</div>
							) : null}
						</>
					)}
					{error ? (
						<p role="alert" className="text-[13px] text-state-alert">
							{error}
						</p>
					) : null}
					<DialogFooter>
						{report !== null ? (
							<Button type="submit">
								{mode === 'table' ? 'Ouvrir la table' : 'Terminer'}
							</Button>
						) : (
							<>
								<Button
									type="button"
									variant="outline"
									onClick={() => onOpenChange(false)}
								>
									Annuler
								</Button>
								<Button
									type="submit"
									disabled={
										isImporting ||
										fileName === null ||
										(mode === 'table' && name.trim() === '')
									}
								>
									{isImporting ? 'Import…' : 'Importer'}
								</Button>
							</>
						)}
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
