import { Unlink } from 'lucide-react'
import { useId, useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	FREQUENCY_OPTIONS,
	type ScheduleRule,
	timezoneOptions,
	WEEKDAY_OPTIONS,
	withFrequency,
} from '#/pages/automation/lib/schedule'

const ORGANIZATION_ZONE = '__organization__'

function FieldMessages({ messages }: { messages: readonly string[] }) {
	return messages.map((message) => (
		<p
			key={message}
			role="alert"
			className="flex items-start gap-1.5 text-state-alert text-xs"
		>
			<Unlink className="mt-0.5 size-3.5 shrink-0" aria-hidden />
			{message}
		</p>
	))
}

export interface ScheduleMessages {
	time: readonly string[]
	weekday: readonly string[]
	timezone: readonly string[]
}

export function ScheduleParameters({
	rule,
	messages,
	onChange,
}: {
	rule: ScheduleRule
	messages: ScheduleMessages
	onChange: (rule: ScheduleRule) => void
}) {
	const baseId = useId()
	const [zonesOpen, setZonesOpen] = useState(false)
	const zones = useMemo(
		() => (zonesOpen ? timezoneOptions(rule.timezone) : [rule.timezone ?? '']),
		[zonesOpen, rule.timezone],
	)

	return (
		<>
			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-frequency`}>Fréquence</Label>
				<Select
					value={rule.frequency}
					onValueChange={(frequency) =>
						onChange(withFrequency(rule, frequency as Schemas.FrequencyDto))
					}
				>
					<SelectTrigger id={`${baseId}-frequency`} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{FREQUENCY_OPTIONS.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
			{rule.frequency === 'Weekly' && (
				<div className="grid gap-2">
					<Label htmlFor={`${baseId}-weekday`}>Jour</Label>
					<Select
						value={rule.weekday ?? 'Monday'}
						onValueChange={(weekday) =>
							onChange({ ...rule, weekday: weekday as Schemas.WeekdayDto })
						}
					>
						<SelectTrigger id={`${baseId}-weekday`} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{WEEKDAY_OPTIONS.map((option) => (
								<SelectItem key={option.value} value={option.value}>
									{option.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
					<FieldMessages messages={messages.weekday} />
				</div>
			)}
			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-time`}>Heure de déclenchement</Label>
				<Input
					id={`${baseId}-time`}
					value={rule.time}
					placeholder="09:00"
					inputMode="numeric"
					autoComplete="off"
					aria-invalid={messages.time.length > 0 || undefined}
					onChange={(event) => onChange({ ...rule, time: event.target.value })}
				/>
				<FieldMessages messages={messages.time} />
				{rule.frequency === 'Hourly' && (
					<p className="text-muted-foreground text-xs">
						Seule la minute compte : à 09:15, le workflow démarre chaque heure à
						la minute 15.
					</p>
				)}
			</div>
			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-timezone`}>Fuseau horaire</Label>
				<Select
					open={zonesOpen}
					onOpenChange={setZonesOpen}
					value={rule.timezone || ORGANIZATION_ZONE}
					onValueChange={(timezone) =>
						onChange({
							...rule,
							timezone: timezone === ORGANIZATION_ZONE ? null : timezone,
						})
					}
				>
					<SelectTrigger id={`${baseId}-timezone`} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={ORGANIZATION_ZONE}>
							Fuseau de l’organisation
						</SelectItem>
						{zones
							.filter((zone) => zone !== '')
							.map((zone) => (
								<SelectItem key={zone} value={zone}>
									{zone}
								</SelectItem>
							))}
					</SelectContent>
				</Select>
				<FieldMessages messages={messages.timezone} />
			</div>
		</>
	)
}
