import type { WorkDay } from '#/pages/hr/lib/rh/types'
import { WorkDayFields } from '#/pages/hr/ui/rh/members/work-day-fields'

export function VariableEditor({
	idPrefix,
	value,
	onChange,
}: {
	idPrefix: string
	value: WorkDay
	onChange: (day: WorkDay) => void
}) {
	return (
		<div className="grid gap-4">
			<p className="rounded-md bg-muted/60 px-3 py-2 text-[13px]">
				Pas de semaine type : tu planifies les jours travaillés dans le
				calendrier du membre.
			</p>
			<section aria-label="Horaires par défaut" className="grid gap-2">
				<div>
					<h4 className="font-medium text-sm">Horaires par défaut</h4>
					<p className="text-[12px] text-muted-foreground">
						Ils préremplissent la planification des jours, tu pourras les
						ajuster à chaque fois.
					</p>
				</div>
				<WorkDayFields
					idPrefix={`${idPrefix}-default`}
					value={value}
					onChange={onChange}
				/>
			</section>
		</div>
	)
}
