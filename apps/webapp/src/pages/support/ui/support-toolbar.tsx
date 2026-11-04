import { X } from 'lucide-react'
import type { FocusEvent, KeyboardEvent } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import type { SupportToolbarProps } from '#/pages/support/types'

export function SupportToolbar({
	filters,
	assignees,
	badges,
	fieldKeys,
	isLoading,
	isFiltered,
	onChange,
	onClear,
}: SupportToolbarProps) {
	const hasFieldKey = Boolean(filters.custom_field_key)

	function commitValue(rawValue: string) {
		const trimmedValue = rawValue.trim()
		const nextValue = trimmedValue === '' ? undefined : trimmedValue
		if (nextValue === filters.custom_field_value) return
		onChange({ custom_field_value: nextValue })
	}

	function handleValueKeyDown(event: KeyboardEvent<HTMLInputElement>) {
		if (event.key !== 'Enter') return
		event.preventDefault()
		commitValue(event.currentTarget.value)
	}

	function handleValueBlur(event: FocusEvent<HTMLInputElement>) {
		commitValue(event.currentTarget.value)
	}

	function handleFieldKeyChange(fieldKey: string) {
		onChange({
			custom_field_key: fieldKey === '' ? undefined : fieldKey,
			custom_field_value: undefined,
		})
	}

	return (
		<div className="flex flex-col gap-3 rounded-md border bg-muted/20 p-3">
			<div className="flex flex-col gap-3 lg:flex-row lg:items-end">
				<div className="space-y-1 lg:w-52">
					<Label
						htmlFor="support-filter-assignee"
						className="text-xs font-semibold"
					>
						Assigné à
					</Label>
					<select
						id="support-filter-assignee"
						className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm disabled:opacity-50"
						disabled={isLoading}
						value={filters.assignee ?? ''}
						onChange={(event) =>
							onChange({ assignee: event.target.value || undefined })
						}
					>
						<option value="">Tout le monde</option>
						{assignees.map((assignee) => (
							<option key={assignee.id} value={assignee.id}>
								{assignee.label}
							</option>
						))}
					</select>
				</div>

				<div className="space-y-1 lg:w-52">
					<Label
						htmlFor="support-filter-badge"
						className="text-xs font-semibold"
					>
						Badge
					</Label>
					<select
						id="support-filter-badge"
						className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm disabled:opacity-50"
						disabled={isLoading}
						value={filters.badge ?? ''}
						onChange={(event) =>
							onChange({ badge: event.target.value || undefined })
						}
					>
						<option value="">Tous les badges</option>
						{badges.map((badge) => (
							<option key={badge.id} value={badge.id}>
								{badge.label}
							</option>
						))}
					</select>
				</div>

				<div className="space-y-1 lg:w-52">
					<Label
						htmlFor="support-filter-field"
						className="text-xs font-semibold"
					>
						Champ
					</Label>
					<select
						id="support-filter-field"
						className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm disabled:opacity-50"
						disabled={isLoading}
						value={filters.custom_field_key ?? ''}
						onChange={(event) => handleFieldKeyChange(event.target.value)}
					>
						<option value="">Tous les champs</option>
						{fieldKeys.map((fieldKey) => (
							<option key={fieldKey} value={fieldKey}>
								{fieldKey}
							</option>
						))}
					</select>
				</div>

				<div className="space-y-1 lg:w-56">
					<Label
						htmlFor="support-filter-value"
						className="text-xs font-semibold"
					>
						Valeur
					</Label>
					<Input
						key={`${filters.custom_field_key ?? ''}:${filters.custom_field_value ?? ''}`}
						id="support-filter-value"
						type="search"
						disabled={!hasFieldKey}
						defaultValue={filters.custom_field_value ?? ''}
						placeholder="Valeur du champ…"
						onKeyDown={handleValueKeyDown}
						onBlur={handleValueBlur}
					/>
				</div>

				{isFiltered ? (
					<Button
						type="button"
						variant="ghost"
						size="sm"
						className="lg:ml-auto"
						onClick={onClear}
					>
						<X className="size-4" />
						Effacer les filtres
					</Button>
				) : null}
			</div>
		</div>
	)
}
