import { cn } from '#/lib/utils'
import {
	EXTRA_BADGE,
	EXTRA_PENDING,
	EXTRA_SOLID,
	HATCH_INACTIVE,
	HATCH_NEUTRAL,
	HOLIDAY_FILL,
	UNPLANNED_FACE,
} from '#/pages/hr/lib/rh/cell-style'
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

export function PlanningLegend({ types }: { types: readonly AbsenceType[] }) {
	return (
		<ul
			aria-label="Légende"
			className="flex shrink-0 flex-wrap items-center gap-x-4 gap-y-1.5 text-muted-foreground text-xs"
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
				<Swatch className={UNPLANNED_FACE} />À planifier
			</li>
			<li className="flex items-center gap-1.5">
				<Swatch className="border border-border" />
				Repos
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
				Heures extra à valider
			</li>
		</ul>
	)
}
