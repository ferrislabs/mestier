import { ListFilter, RotateCcw } from 'lucide-react'
import {
	type FocusEvent,
	type KeyboardEvent,
	type ReactNode,
	useId,
} from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	ALL_FILTER,
	activeFilterCount,
	filterPatchOf,
	filterValuesOf,
} from '#/pages/support/lib/ticket-filters'
import type { SupportToolbarProps } from '#/pages/support/types'
import {
	AssigneeFilterSelect,
	BadgeFilterSelect,
	PriorityFilterSelect,
	StatusFilterSelect,
} from '#/pages/support/ui/ticket-filter-selects'

function Field({
	label,
	htmlFor,
	children,
}: {
	label: string
	htmlFor: string
	children: ReactNode
}) {
	return (
		<div className="grid gap-1.5">
			<Label htmlFor={htmlFor}>{label}</Label>
			{children}
		</div>
	)
}

export type TicketFiltersPopoverProps = Pick<
	SupportToolbarProps,
	| 'filters'
	| 'members'
	| 'assigneeCounts'
	| 'badges'
	| 'fieldKeys'
	| 'isLoading'
	| 'showStatus'
	| 'onChange'
	| 'onReset'
>

export function TicketFiltersPopover({
	filters,
	members,
	assigneeCounts,
	badges,
	fieldKeys,
	isLoading,
	showStatus,
	onChange,
	onReset,
}: TicketFiltersPopoverProps) {
	const baseId = useId()
	const values = filterValuesOf(filters)
	const hasCustomField = Boolean(
		filters.custom_field_key && filters.custom_field_value,
	)
	const count = activeFilterCount(values, showStatus) + Number(hasCustomField)

	function commitFieldValue(rawValue: string) {
		const trimmed = rawValue.trim()
		const next = trimmed === '' ? undefined : trimmed
		if (next === filters.custom_field_value) return
		onChange({ custom_field_value: next })
	}

	function handleFieldValueKeyDown(event: KeyboardEvent<HTMLInputElement>) {
		if (event.key !== 'Enter') return
		event.preventDefault()
		commitFieldValue(event.currentTarget.value)
	}

	function handleFieldValueBlur(event: FocusEvent<HTMLInputElement>) {
		commitFieldValue(event.currentTarget.value)
	}

	return (
		<Popover>
			<PopoverTrigger asChild>
				<Button
					variant="outline"
					className={count > 0 ? 'border-ring/60 bg-accent' : undefined}
				>
					<ListFilter />
					Filtres
					{count > 0 && (
						<span
							className="rounded-full bg-primary px-1.5 font-semibold text-[11px] text-primary-foreground leading-4 tabular-nums"
							role="img"
							aria-label={`${count} filtre${count > 1 ? 's' : ''} actif${count > 1 ? 's' : ''}`}
						>
							{count}
						</span>
					)}
				</Button>
			</PopoverTrigger>
			<PopoverContent align="end" className="grid w-72 gap-4">
				<div className="flex items-center justify-between gap-2">
					<h2 className="font-semibold text-sm">Filtres</h2>
					<Button
						type="button"
						variant="ghost"
						size="sm"
						className="-mr-2 h-7 text-muted-foreground"
						disabled={count === 0}
						onClick={onReset}
					>
						<RotateCcw />
						Réinitialiser
					</Button>
				</div>
				<Field label="Assigné à" htmlFor={`${baseId}-assignee`}>
					<AssigneeFilterSelect
						id={`${baseId}-assignee`}
						value={values.assignee}
						onChange={(assignee) => onChange(filterPatchOf({ assignee }))}
						counts={assigneeCounts}
						members={members}
						disabled={isLoading}
						className="w-full min-w-0"
					/>
				</Field>
				{showStatus && (
					<Field label="Statut" htmlFor={`${baseId}-status`}>
						<StatusFilterSelect
							id={`${baseId}-status`}
							value={values.status}
							onChange={(status) => onChange(filterPatchOf({ status }))}
							className="w-full min-w-0"
						/>
					</Field>
				)}
				<Field label="Priorité" htmlFor={`${baseId}-priority`}>
					<PriorityFilterSelect
						id={`${baseId}-priority`}
						value={values.priority}
						onChange={(priority) => onChange(filterPatchOf({ priority }))}
						className="w-full min-w-0"
					/>
				</Field>
				<Field label="Badge" htmlFor={`${baseId}-badge`}>
					<BadgeFilterSelect
						id={`${baseId}-badge`}
						value={values.badge}
						onChange={(badge) => onChange(filterPatchOf({ badge }))}
						badges={badges}
						className="w-full min-w-0"
					/>
				</Field>
				<Field label="Champ" htmlFor={`${baseId}-field`}>
					<Select
						value={filters.custom_field_key ?? ALL_FILTER}
						disabled={isLoading}
						onValueChange={(next) =>
							onChange({
								custom_field_key: next === ALL_FILTER ? undefined : next,
								custom_field_value: undefined,
							})
						}
					>
						<SelectTrigger id={`${baseId}-field`} className="w-full min-w-0">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value={ALL_FILTER}>Tous les champs</SelectItem>
							{fieldKeys.map((fieldKey) => (
								<SelectItem key={fieldKey} value={fieldKey}>
									{fieldKey}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</Field>
				<Field label="Valeur" htmlFor={`${baseId}-value`}>
					<Input
						key={`${filters.custom_field_key ?? ''}:${filters.custom_field_value ?? ''}`}
						id={`${baseId}-value`}
						type="search"
						disabled={!filters.custom_field_key}
						defaultValue={filters.custom_field_value ?? ''}
						placeholder="Valeur du champ…"
						onKeyDown={handleFieldValueKeyDown}
						onBlur={handleFieldValueBlur}
					/>
				</Field>
			</PopoverContent>
		</Popover>
	)
}
