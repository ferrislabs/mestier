import { Check, ChevronDown } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import type { FilterOption } from '#/pages/cursus/shared/lib/list-filters'

export interface SearchSelectButtonProps<Value extends string> {
	value: Value
	label: string
	options: readonly FilterOption<Value>[]
	onChange: (value: Value) => void
}

export function SearchSelectButton<Value extends string>({
	value,
	label,
	options,
	onChange,
}: SearchSelectButtonProps<Value>) {
	const [open, setOpen] = useState(false)
	const current = options.find((option) => option.value === value)

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button
					type="button"
					variant="outline"
					size="sm"
					role="combobox"
					aria-expanded={open}
					aria-label={label}
					className="w-52 justify-between bg-card font-normal"
				>
					<span className="truncate">{current?.label ?? label}</span>
					<ChevronDown className="opacity-50" aria-hidden />
				</Button>
			</PopoverTrigger>
			<PopoverContent align="start" className="w-64 p-0">
				<Command>
					<CommandInput placeholder="Rechercher…" />
					<CommandList>
						<CommandEmpty>Aucun résultat.</CommandEmpty>
						<CommandGroup>
							{options.map((option) => (
								<CommandItem
									key={option.value}
									value={`${option.label} ${option.value}`}
									onSelect={() => {
										onChange(option.value)
										setOpen(false)
									}}
								>
									<span className="min-w-0 flex-1 truncate">
										{option.label}
									</span>
									<Check
										className={cn(
											'size-4',
											option.value === value ? 'opacity-100' : 'opacity-0',
										)}
										aria-hidden
									/>
								</CommandItem>
							))}
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	)
}
