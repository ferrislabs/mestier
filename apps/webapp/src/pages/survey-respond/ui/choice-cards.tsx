import { Check } from 'lucide-react'
import { cn } from '#/lib/utils'

export function ChoiceCards({
	name,
	options,
	multiple,
	selected,
	invalid,
	describedBy,
	labelledBy,
	onChange,
}: {
	name: string
	options: readonly string[]
	multiple: boolean
	selected: readonly string[]
	invalid: boolean
	describedBy: string | undefined
	labelledBy: string
	onChange: (selected: string[]) => void
}) {
	return (
		<fieldset
			aria-labelledby={labelledBy}
			aria-describedby={describedBy}
			className="m-0 grid min-w-0 gap-3 border-0 p-0"
		>
			{options.map((option, position) => {
				const checked = selected.includes(option)
				const id = `${name}-${position}`
				return (
					<label
						key={option}
						htmlFor={id}
						className={cn(
							'group flex min-h-14 cursor-pointer items-center gap-4 rounded-xl border bg-card px-4 py-3 text-base transition-colors',
							'hover:border-primary/50 has-[:focus-visible]:ring-[3px] has-[:focus-visible]:ring-ring/50',
							checked && 'border-primary bg-primary/5',
							invalid && !checked && 'border-destructive/60',
						)}
					>
						<input
							id={id}
							type={multiple ? 'checkbox' : 'radio'}
							name={name}
							value={option}
							checked={checked}
							className="sr-only"
							onChange={() =>
								onChange(
									multiple
										? checked
											? selected.filter((item) => item !== option)
											: options.filter(
													(item) => item === option || selected.includes(item),
												)
										: [option],
								)
							}
						/>
						<span
							aria-hidden
							className={cn(
								'flex size-6 shrink-0 items-center justify-center border-2 transition-colors',
								multiple ? 'rounded-md' : 'rounded-full',
								checked
									? 'border-primary bg-primary text-primary-foreground'
									: 'border-muted-foreground/40 bg-background',
							)}
						>
							{checked ? <Check className="size-4" strokeWidth={3} /> : null}
						</span>
						<span className="min-w-0 flex-1 break-words">{option}</span>
					</label>
				)
			})}
		</fieldset>
	)
}
