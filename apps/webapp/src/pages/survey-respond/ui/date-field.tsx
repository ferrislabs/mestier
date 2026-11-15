import { format, isValid, parse } from 'date-fns'
import { fr } from 'date-fns/locale'
import { CalendarIcon } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Calendar } from '#/components/ui/calendar'
import { Input } from '#/components/ui/input'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'

const DAY = 'yyyy-MM-dd'

function parseDay(value: string): Date | undefined {
	const parsed = parse(value.slice(0, 10), DAY, new Date())
	return isValid(parsed) ? parsed : undefined
}

export function DateField({
	id,
	withTime,
	value,
	invalid,
	describedBy,
	onChange,
}: {
	id: string
	withTime: boolean
	value: string
	invalid: boolean
	describedBy: string | undefined
	onChange: (value: string) => void
}) {
	const day = parseDay(value)
	const time = withTime ? value.slice(11, 16) : ''
	const compose = (nextDay: Date, nextTime: string) =>
		withTime
			? `${format(nextDay, DAY)}T${nextTime || '00:00'}`
			: format(nextDay, DAY)
	return (
		<div className="flex flex-wrap gap-2">
			<Popover>
				<PopoverTrigger asChild>
					<Button
						id={id}
						type="button"
						variant="outline"
						aria-invalid={invalid || undefined}
						aria-describedby={describedBy}
						className={cn(
							'h-10 min-w-0 flex-1 justify-start rounded-md bg-card px-3 text-sm font-normal shadow-none',
							!day && 'text-muted-foreground',
						)}
					>
						<CalendarIcon aria-hidden />
						{day
							? format(day, 'd MMMM yyyy', { locale: fr })
							: 'Choisir une date'}
					</Button>
				</PopoverTrigger>
				<PopoverContent className="w-auto p-0" align="start">
					<Calendar
						mode="single"
						locale={fr}
						captionLayout="dropdown"
						selected={day}
						defaultMonth={day}
						onSelect={(selected) => {
							if (selected) onChange(compose(selected, time))
						}}
					/>
				</PopoverContent>
			</Popover>
			{withTime ? (
				<Input
					type="time"
					aria-label="Heure"
					value={time}
					aria-invalid={invalid || undefined}
					className="h-10 w-28 rounded-md bg-card px-3 text-sm shadow-none md:text-sm [&::-webkit-calendar-picker-indicator]:hidden"
					onChange={(event) =>
						onChange(compose(day ?? new Date(), event.target.value))
					}
				/>
			) : null}
		</div>
	)
}
