import { cn } from '#/lib/utils'
import {
	EXTRA_BADGE,
	EXTRA_PENDING,
	EXTRA_SOLID,
	HATCH_INACTIVE,
	HATCH_NEUTRAL,
	HOLIDAY_FILL,
} from '#/pages/hr/lib/rh/calendar-style'
import type { AbsenceType } from '#/pages/hr/lib/rh/types'
import { TypeDot } from '#/pages/hr/ui/rh/shared/type-chip'

function Swatch({ className }: { className: string }) {
	return (
		<span
			aria-hidden
			className={cn('inline-block h-3 w-5 shrink-0 rounded-[3px]', className)}
		/>
	)
}

export function CalendarLegend({ types }: { types: readonly AbsenceType[] }) {
	return (
		<ul
			aria-label="Légende du calendrier"
			className="flex flex-wrap items-center gap-x-4 gap-y-1.5 text-muted-foreground text-xs"
		>
			{types
				.filter((type) => !type.archived)
				.map((type) => (
					<li key={type.id} className="flex items-center gap-1.5">
						<TypeDot hue={type.hue} />
						{type.name}
					</li>
				))}
			<li className="flex items-center gap-1.5">
				<Swatch
					className={cn(
						'border border-muted-foreground/60 border-dashed',
						HATCH_NEUTRAL,
					)}
				/>
				À valider
			</li>
			<li className="flex items-center gap-1.5">
				<Swatch className={HOLIDAY_FILL} />
				Férié
			</li>
			<li className="flex items-center gap-1.5">
				<Swatch className={cn('border border-border/60', HATCH_INACTIVE)} />
				Hors contrat
			</li>
			<li className="flex items-center gap-1.5">
				<Swatch className="border border-muted-foreground/50 border-dashed" />À
				planifier
			</li>
			<li className="flex items-center gap-1.5">
				<Swatch className="bg-muted/70" />
				Séquence sans travail
			</li>
			<li className="flex items-center gap-1.5">
				<span aria-hidden className="size-1.5 rounded-full bg-state-today" />
				Jour modifié
			</li>
			<li className="flex items-center gap-1.5">
				<span aria-hidden className={cn(EXTRA_BADGE, EXTRA_SOLID)}>
					+1 h
				</span>
				Heures extra
			</li>
			<li className="flex items-center gap-1.5">
				<span aria-hidden className={cn(EXTRA_BADGE, EXTRA_PENDING)}>
					+1 h
				</span>
				À valider
			</li>
			<li className="basis-full">
				Journée coupée en deux : en haut le matin, en bas l’après-midi.
			</li>
		</ul>
	)
}
