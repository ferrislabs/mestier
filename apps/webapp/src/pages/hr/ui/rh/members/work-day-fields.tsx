import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Switch } from '#/components/ui/switch'
import { dayHoursLabel } from '#/pages/hr/lib/rh/format'
import { validateWorkDay } from '#/pages/hr/lib/rh/schedule'
import {
	patchLunch,
	patchWorkDay,
	toggleLunch,
} from '#/pages/hr/lib/rh/schedule-draft'
import type { WorkDay } from '#/pages/hr/lib/rh/types'

const TIME = 'h-8 w-36 shrink-0 bg-card'

export function WorkDayFields({
	idPrefix,
	value,
	onChange,
}: {
	idPrefix: string
	value: WorkDay
	onChange: (day: WorkDay) => void
}) {
	const error = validateWorkDay(value)
	const errorId = `${idPrefix}-error`
	const lunchId = `${idPrefix}-lunch`
	const invalid = error ? true : undefined
	const lunchError = Boolean(error?.includes('pause'))
	return (
		<div className="grid gap-2">
			<div className="flex flex-wrap items-center gap-3">
				<Input
					type="time"
					value={value.start}
					onChange={(event) =>
						onChange(patchWorkDay(value, { start: event.target.value }))
					}
					aria-label="Début de journée"
					aria-invalid={lunchError ? undefined : invalid}
					aria-describedby={error ? errorId : undefined}
					className={TIME}
				/>
				<span aria-hidden className="text-muted-foreground">
					–
				</span>
				<Input
					type="time"
					value={value.end}
					onChange={(event) =>
						onChange(patchWorkDay(value, { end: event.target.value }))
					}
					aria-label="Fin de journée"
					aria-invalid={lunchError ? undefined : invalid}
					aria-describedby={error ? errorId : undefined}
					className={TIME}
				/>
				{!error && (
					<span className="tabular text-[13px] text-muted-foreground">
						{dayHoursLabel(value)} nettes
					</span>
				)}
			</div>
			<div className="flex flex-wrap items-center gap-3">
				<Switch
					id={lunchId}
					size="sm"
					checked={Boolean(value.lunch)}
					onCheckedChange={(enabled) => onChange(toggleLunch(value, enabled))}
					aria-label="Pause déjeuner"
				/>
				<Label
					htmlFor={lunchId}
					className="font-normal text-[13px] text-muted-foreground"
				>
					Pause
				</Label>
				{value.lunch ? (
					<>
						<Input
							type="time"
							value={value.lunch.start}
							onChange={(event) =>
								onChange(patchLunch(value, { start: event.target.value }))
							}
							aria-label="Début de pause"
							aria-invalid={lunchError ? invalid : undefined}
							aria-describedby={error ? errorId : undefined}
							className={TIME}
						/>
						<span aria-hidden className="text-muted-foreground">
							–
						</span>
						<Input
							type="time"
							value={value.lunch.end}
							onChange={(event) =>
								onChange(patchLunch(value, { end: event.target.value }))
							}
							aria-label="Fin de pause"
							aria-invalid={lunchError ? invalid : undefined}
							aria-describedby={error ? errorId : undefined}
							className={TIME}
						/>
					</>
				) : (
					<span className="text-[13px] text-muted-foreground">Sans pause</span>
				)}
			</div>
			{error && (
				<p id={errorId} className="text-[12px] text-state-alert">
					{error}
				</p>
			)}
		</div>
	)
}
