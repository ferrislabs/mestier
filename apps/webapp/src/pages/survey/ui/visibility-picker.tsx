import { Globe, Users } from 'lucide-react'
import { cn } from '#/lib/utils'
import { VISIBILITY } from '#/pages/survey/lib/survey-format'
import type { Visibility } from '#/pages/survey/types'

const OPTIONS: { value: Visibility; icon: typeof Globe }[] = [
	{ value: 'PUBLIC', icon: Globe },
	{ value: 'MEMBERS', icon: Users },
]

export function VisibilityPicker({
	name,
	value,
	disabled = false,
	onChange,
}: {
	name: string
	value: Visibility
	disabled?: boolean
	onChange: (value: Visibility) => void
}) {
	return (
		<fieldset className="grid gap-2 sm:grid-cols-2" disabled={disabled}>
			<legend className="sr-only">Visibilité</legend>
			{OPTIONS.map(({ value: option, icon: Icon }) => (
				<label
					key={option}
					className={cn(
						'flex cursor-pointer gap-3 rounded-lg border p-3 text-sm transition-colors has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring',
						value === option ? 'border-primary bg-brand-soft' : 'bg-card',
						disabled && 'cursor-not-allowed opacity-70',
					)}
				>
					<input
						type="radio"
						name={name}
						value={option}
						checked={value === option}
						onChange={() => onChange(option)}
						className="sr-only"
					/>
					<Icon className="mt-0.5 size-4 shrink-0" aria-hidden />
					<span>
						<span className="block font-medium">
							{VISIBILITY[option].label}
						</span>
						<span className="block text-muted-foreground text-xs">
							{VISIBILITY[option].description}
						</span>
					</span>
				</label>
			))}
		</fieldset>
	)
}
