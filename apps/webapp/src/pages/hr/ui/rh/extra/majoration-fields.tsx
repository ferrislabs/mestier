import { useId } from 'react'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectSeparator,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { formatRate, MAX_RATE } from '#/pages/hr/lib/rh/extra'
import { CUSTOM_RATE, type Majoration } from '#/pages/hr/lib/rh/extra-view'
import type { Surcharge } from '#/pages/hr/lib/rh/types'
import { SwitchRow } from '#/pages/hr/ui/rh/extra/switch-row'

export function MajorationFields({
	surcharges,
	majoration,
	suggested,
	errorId,
	invalid,
	onChange,
}: {
	surcharges: readonly Surcharge[]
	majoration: Majoration
	suggested: boolean
	errorId: string
	invalid: boolean
	onChange: (patch: Partial<Majoration>) => void
}) {
	const baseId = useId()
	const choiceId = `${baseId}-choice`
	const rateId = `${baseId}-rate`
	return (
		<div className="grid gap-3 rounded-lg border p-3">
			<SwitchRow
				label="Appliquer une majoration"
				help={
					majoration.enabled
						? 'Le taux est figé à la déclaration.'
						: 'Les heures sont comptées sans majoration.'
				}
				checked={majoration.enabled}
				onChange={(enabled) => onChange({ enabled })}
			/>
			{majoration.enabled && (
				<div className="grid gap-3 border-t pt-3">
					<div className="grid gap-1.5">
						<Label htmlFor={choiceId}>Barème</Label>
						<Select
							value={majoration.choice}
							onValueChange={(choice) => onChange({ choice })}
						>
							<SelectTrigger id={choiceId} className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{surcharges.map((surcharge) => (
									<SelectItem key={surcharge.id} value={surcharge.id}>
										{surcharge.name} · {formatRate(surcharge.rate)}
									</SelectItem>
								))}
								{surcharges.length > 0 && <SelectSeparator />}
								<SelectItem value={CUSTOM_RATE}>Autre taux</SelectItem>
							</SelectContent>
						</Select>
						{suggested && majoration.choice !== CUSTOM_RATE && (
							<p className="text-muted-foreground text-xs">
								Barème proposé d’après la date, modifiable.
							</p>
						)}
					</div>
					{majoration.choice === CUSTOM_RATE && (
						<div className="grid gap-1.5">
							<Label htmlFor={rateId}>Taux de majoration</Label>
							<div className="relative w-40">
								<Input
									id={rateId}
									type="number"
									inputMode="decimal"
									min={0}
									max={MAX_RATE}
									step="any"
									value={majoration.custom}
									placeholder="30"
									aria-required
									aria-invalid={invalid ? true : undefined}
									aria-describedby={invalid ? errorId : undefined}
									onChange={(event) => onChange({ custom: event.target.value })}
									className="pr-8"
								/>
								<span
									aria-hidden
									className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-muted-foreground text-sm"
								>
									%
								</span>
							</div>
						</div>
					)}
				</div>
			)}
		</div>
	)
}
