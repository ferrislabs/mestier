import { ChevronDown } from 'lucide-react'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuCheckboxItem,
	DropdownMenuContent,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import type { FilterOption } from '#/pages/cursus/shared/lib/list-filters'

export interface MultiFilterButtonProps<Value extends string> {
	name: string
	options: readonly FilterOption<Value>[]
	selected: readonly Value[]
	onChange: (selected: Value[]) => void
}

export function MultiFilterButton<Value extends string>({
	name,
	options,
	selected,
	onChange,
}: MultiFilterButtonProps<Value>) {
	const chosen = options.filter((option) => selected.includes(option.value))
	const [first] = chosen

	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button
					type="button"
					variant="outline"
					size="sm"
					className="bg-card"
					aria-label={name}
				>
					{chosen.length === 1 && first ? first.label : name}
					{chosen.length > 1 ? (
						<Badge variant="secondary" className="px-1.5 py-0">
							{chosen.length}
						</Badge>
					) : null}
					<ChevronDown aria-hidden />
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="start">
				{options.map((option) => (
					<DropdownMenuCheckboxItem
						key={option.value}
						checked={selected.includes(option.value)}
						onSelect={(event) => event.preventDefault()}
						onCheckedChange={(checked) =>
							onChange(
								options
									.map((candidate) => candidate.value)
									.filter((value) =>
										value === option.value ? checked : selected.includes(value),
									),
							)
						}
					>
						{option.label}
					</DropdownMenuCheckboxItem>
				))}
			</DropdownMenuContent>
		</DropdownMenu>
	)
}
