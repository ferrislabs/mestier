import { Search } from 'lucide-react'
import type { ReactNode } from 'react'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { FilterOption } from '#/pages/cursus/shared/lib/list-filters'
import { MultiFilterButton } from '#/pages/cursus/shared/ui/multi-filter-button'

export interface ListToolbarSelect<Value extends string> {
	value: Value
	label: string
	options: readonly FilterOption<Value>[]
	onChange: (value: Value) => void
}

export interface ListToolbarProps<Status extends string, Value extends string> {
	search: string
	searchPlaceholder: string
	onSearch: (search: string) => void
	status: readonly Status[]
	statusOptions: readonly FilterOption<Status>[]
	statusLabel: string
	onStatus: (status: Status[]) => void
	select?: ListToolbarSelect<Value>
	extra?: ReactNode
}

export function ListToolbar<Status extends string, Value extends string>({
	search,
	searchPlaceholder,
	onSearch,
	status,
	statusOptions,
	statusLabel,
	onStatus,
	select,
	extra,
}: ListToolbarProps<Status, Value>) {
	return (
		<div className="mb-3 flex flex-wrap items-center gap-2">
			<div className="relative w-full max-w-xs">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					value={search}
					onChange={(event) => onSearch(event.target.value)}
					placeholder={searchPlaceholder}
					aria-label={searchPlaceholder}
					className="h-8 bg-card pl-8"
				/>
			</div>
			<MultiFilterButton
				name={statusLabel}
				options={statusOptions}
				selected={status}
				onChange={onStatus}
			/>
			{extra}
			{select ? (
				<Select
					value={select.value}
					onValueChange={(value) => {
						const next = select.options.find((option) => option.value === value)
						if (next) select.onChange(next.value)
					}}
				>
					<SelectTrigger
						size="sm"
						className="w-52 bg-card"
						aria-label={select.label}
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{select.options.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			) : null}
		</div>
	)
}
