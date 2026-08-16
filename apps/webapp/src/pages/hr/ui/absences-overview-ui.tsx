import {
	CalendarOff,
	Loader2,
	MoreHorizontal,
	Plus,
	Trash2,
} from 'lucide-react'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
	AlertDialogTrigger,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import {
	ABSENCE_KIND_LABELS,
	type AbsenceFormValues,
	type AbsenceKind,
} from '#/pages/hr/lib/absences'
import { formatDateFr } from '#/pages/hr/types'
import {
	AbsenceFormSheet,
	type AbsenceFormSheetProps,
} from '#/pages/hr/ui/absence-form-sheet'

const ABSENCE_KIND_TONE: Record<AbsenceKind, 'neutral' | 'warning' | 'brand'> =
	{
		LEAVE: 'brand',
		SICK: 'warning',
		UNAVAILABLE: 'neutral',
	}

/** One row of the org-wide absence list — the shared draft shape plus the absence's own id and the resolved member name (built by the feature, see {@link AbsencesOverviewFeature}). */
export type AbsenceOverviewRow = AbsenceFormValues & {
	id: string
	memberDisplayName: string
}

export interface AbsencesOverviewUIProps {
	organizationName: string
	isLoading: boolean
	error: string | null
	absences: AbsenceOverviewRow[]
	onCreate: () => void
	onEdit: (absenceId: string) => void
	onDelete: (absenceId: string) => Promise<unknown>
	/** The create/edit form, reused as-is from the member work-time screen (see {@link AbsenceFormSheet}). */
	absenceSheet: AbsenceFormSheetProps
}

/**
 * Every absence across the organization, one flat list — complementary to
 * the per-member absences section on `EmployeeWorkTimeUI` and to the
 * planning calendar's own inline editing, neither of which this page
 * replaces (see the "Équipe & ressources" reorg design doc).
 */
export function AbsencesOverviewUI({
	organizationName,
	isLoading,
	error,
	absences,
	onCreate,
	onEdit,
	onDelete,
	absenceSheet,
}: AbsencesOverviewUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Absences"
				description="Congés, arrêts et indisponibilités de toute l’organisation. L’assignation d’un chantier pendant cette période n’est jamais bloquée, seulement signalée."
				actions={
					<Button onClick={onCreate}>
						<Plus />
						Ajouter une absence
					</Button>
				}
			/>

			<MetricCard
				label="Absences"
				value={absences.length}
				hint="Toutes personnes confondues"
				icon={<CalendarOff className="size-4" />}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<AbsencesOverviewUI.Loading />
			) : (
				<AbsencesTable data={absences} onEdit={onEdit} onDelete={onDelete} />
			)}

			<AbsenceFormSheet {...absenceSheet} />
		</PageShell>
	)
}

AbsencesOverviewUI.Loading = function AbsencesOverviewLoading() {
	return (
		<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
			<Loader2 className="size-5 animate-spin" />
			Chargement des absences…
		</SectionCard>
	)
}

interface AbsencesTableProps {
	data: AbsenceOverviewRow[]
	onEdit: (absenceId: string) => void
	onDelete: (absenceId: string) => Promise<unknown>
}

function AbsencesTable({ data, onEdit, onDelete }: AbsencesTableProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Absences (${data.length})`}
				description="Nature, période et note de chaque absence, tous membres confondus."
			/>
			<div className="overflow-x-auto">
				<table className="w-full min-w-[720px] border-collapse text-sm">
					<thead>
						<tr className="border-b bg-muted/50">
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Personne
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Nature
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Période
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Note
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								<span className="sr-only">Actions</span>
							</th>
						</tr>
					</thead>
					<tbody>
						{data.length === 0 ? (
							<tr>
								<td colSpan={5} className="px-5 py-12 text-center">
									<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
										<p className="font-medium">Aucune absence trouvée</p>
										<p className="text-sm text-muted-foreground">
											Ajoutez une absence pour la signaler sur le planning de
											l’équipe.
										</p>
									</div>
								</td>
							</tr>
						) : (
							data.map((absence) => (
								<tr
									key={absence.id}
									className="group border-b transition hover:bg-muted/35 hover:shadow-xs last:border-b-0"
								>
									<td className="px-5 py-3 align-middle">
										<p className="truncate font-medium">
											{absence.memberDisplayName}
										</p>
									</td>
									<td className="px-5 py-3 align-middle">
										<StatusBadge tone={ABSENCE_KIND_TONE[absence.kind]}>
											{ABSENCE_KIND_LABELS[absence.kind]}
										</StatusBadge>
									</td>
									<td className="px-5 py-3 align-middle">
										{formatAbsenceRange(absence)}
									</td>
									<td className="px-5 py-3 align-middle">
										{absence.note ? (
											<span className="text-muted-foreground">
												{absence.note}
											</span>
										) : (
											<span className="text-muted-foreground italic">
												Aucune
											</span>
										)}
									</td>
									<td className="px-5 py-3 align-middle">
										<RowActions
											memberDisplayName={absence.memberDisplayName}
											onEdit={() => onEdit(absence.id)}
											onDelete={() => onDelete(absence.id)}
										/>
									</td>
								</tr>
							))
						)}
					</tbody>
				</table>
			</div>
		</SectionCard>
	)
}

interface RowActionsProps {
	memberDisplayName: string
	onEdit: () => void
	onDelete: () => Promise<unknown>
}

function RowActions({ memberDisplayName, onEdit, onDelete }: RowActionsProps) {
	return (
		<AlertDialog>
			<div className="flex justify-end opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<Button size="icon-sm" variant="ghost">
							<MoreHorizontal />
							<span className="sr-only">Actions</span>
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end">
						<DropdownMenuItem onClick={onEdit}>Modifier</DropdownMenuItem>
						<DropdownMenuSeparator />
						<AlertDialogTrigger asChild>
							<DropdownMenuItem
								variant="destructive"
								onSelect={(event) => event.preventDefault()}
							>
								<Trash2 />
								Supprimer
							</DropdownMenuItem>
						</AlertDialogTrigger>
					</DropdownMenuContent>
				</DropdownMenu>
			</div>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>
						Supprimer l’absence de {memberDisplayName} ?
					</AlertDialogTitle>
					<AlertDialogDescription>
						Cette absence sera définitivement supprimée. Cette action est
						irréversible.
					</AlertDialogDescription>
				</AlertDialogHeader>
				<AlertDialogFooter>
					<AlertDialogCancel>Annuler</AlertDialogCancel>
					<AlertDialogAction onClick={onDelete}>Supprimer</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}

function formatAbsenceRange(item: AbsenceFormValues): string {
	const sameDay = item.range.from === item.range.to
	const days = sameDay
		? formatDateFr(item.range.from)
		: `${formatDateFr(item.range.from)} → ${formatDateFr(item.range.to)}`

	if (item.allDay) return days

	return sameDay
		? `${days} ${item.startTime}–${item.endTime}`
		: `${formatDateFr(item.range.from)} ${item.startTime} → ${formatDateFr(item.range.to)} ${item.endTime}`
}
