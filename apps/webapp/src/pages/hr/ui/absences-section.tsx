import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import {
	ABSENCE_KIND_LABELS,
	type AbsenceListItem,
} from '#/pages/hr/lib/absences'
import { formatDateFr } from '#/pages/hr/types'

export interface AbsencesSectionProps {
	absences: AbsenceListItem[]
	isLoading: boolean
	onCreate: () => void
	onSelect: (absenceId: string) => void
}

/**
 * The third pillar of the employee's work-time screen, alongside the
 * rhythm and work-slots sections — see the planning remodel design doc's
 * "Absences vers le module RH" decision: an absence is a fact about the
 * employee, so it lives here rather than on the planning grid, which keeps
 * displaying it but no longer edits it.
 */
export function AbsencesSection({
	absences,
	isLoading,
	onCreate,
	onSelect,
}: AbsencesSectionProps) {
	return (
		<SectionCard>
			<SectionHeader
				title="Absences"
				description="Congés, arrêts et indisponibilités de cet employé. L’assignation d’un projet pendant cette période n’est jamais bloquée, seulement signalée."
			/>
			<div className="flex flex-col gap-4 p-5">
				{isLoading ? (
					<p className="text-sm text-muted-foreground">
						Chargement des absences…
					</p>
				) : absences.length === 0 ? (
					<p className="text-sm text-muted-foreground">
						Aucune absence enregistrée.
					</p>
				) : (
					<ul className="flex flex-col divide-y rounded-lg border">
						{absences.map((absence) => (
							<li key={absence.id}>
								<button
									type="button"
									className="flex w-full flex-col gap-0.5 px-4 py-3 text-left hover:bg-muted/50 sm:flex-row sm:items-center sm:justify-between"
									onClick={() => onSelect(absence.id)}
								>
									<span className="text-sm font-medium">
										{ABSENCE_KIND_LABELS[absence.kind]} —{' '}
										{formatAbsenceRange(absence)}
									</span>
									{absence.note ? (
										<span className="text-xs text-muted-foreground">
											{absence.note}
										</span>
									) : null}
								</button>
							</li>
						))}
					</ul>
				)}

				<div>
					<Button type="button" size="sm" variant="outline" onClick={onCreate}>
						<Plus />
						Ajouter une absence
					</Button>
				</div>
			</div>
		</SectionCard>
	)
}

function formatAbsenceRange(item: AbsenceListItem): string {
	const sameDay = item.range.from === item.range.to
	const days = sameDay
		? formatDateFr(item.range.from)
		: `${formatDateFr(item.range.from)} → ${formatDateFr(item.range.to)}`

	if (item.allDay) return days

	return sameDay
		? `${days} ${item.startTime}–${item.endTime}`
		: `${formatDateFr(item.range.from)} ${item.startTime} → ${formatDateFr(item.range.to)} ${item.endTime}`
}
