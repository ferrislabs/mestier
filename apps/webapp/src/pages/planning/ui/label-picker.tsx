import { Check, Plus, Tag } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import { matchLabelByName } from '#/pages/planning/lib/labels'

export interface LabelPickerOption {
	id: string
	name: string
	color: string
}

export interface LabelPickerProps {
	labels: LabelPickerOption[]
	selectedIds: string[]
	isCreating?: boolean
	onToggle: (labelId: string) => void
	onCreate: (name: string) => void
}

/**
 * The label multi-select on the task form — pure presentation, all state
 * (the organization's labels, the current selection, the create mutation)
 * comes in as props. Typing a name that matches nothing existing offers to
 * create it right there (see the planning remodel design doc's "Création de
 * label" decision) rather than sending the user to a separate screen.
 */
export function LabelPicker({
	labels,
	selectedIds,
	isCreating,
	onToggle,
	onCreate,
}: LabelPickerProps) {
	const [query, setQuery] = useState('')

	const selected = labels.filter((label) => selectedIds.includes(label.id))
	const normalizedQuery = query.trim().toLowerCase()
	const filtered = normalizedQuery
		? labels.filter((label) =>
				label.name.toLowerCase().includes(normalizedQuery),
			)
		: labels
	const exactMatch = matchLabelByName(labels, query)
	const canCreate = query.trim().length > 0 && !exactMatch

	return (
		<Popover>
			<PopoverTrigger asChild>
				<Button
					type="button"
					variant="outline"
					className="h-auto min-h-9 w-full flex-wrap justify-start gap-1.5 py-1.5"
				>
					<Tag className="size-4 shrink-0 text-muted-foreground" />
					{selected.length === 0 ? (
						<span className="text-muted-foreground">Aucun label</span>
					) : (
						selected.map((label) => (
							<span
								key={label.id}
								className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium text-white"
								style={{ backgroundColor: label.color }}
							>
								{label.name}
							</span>
						))
					)}
				</Button>
			</PopoverTrigger>
			<PopoverContent className="w-72 p-2" align="start">
				<Input
					value={query}
					onChange={(event) => setQuery(event.target.value)}
					placeholder="Rechercher ou créer…"
					aria-label="Rechercher ou créer un label"
					autoFocus
				/>

				<div
					role="listbox"
					aria-label="Labels"
					className="mt-2 flex flex-col gap-0.5"
				>
					{filtered.map((label) => {
						const isSelected = selectedIds.includes(label.id)
						return (
							<button
								key={label.id}
								type="button"
								role="option"
								aria-selected={isSelected}
								onClick={() => onToggle(label.id)}
								className={cn(
									'flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent',
									isSelected && 'bg-accent/60',
								)}
							>
								<span
									className="size-2.5 shrink-0 rounded-full"
									style={{ backgroundColor: label.color }}
								/>
								<span className="flex-1 truncate">{label.name}</span>
								{isSelected ? <Check className="size-4 shrink-0" /> : null}
							</button>
						)
					})}
					{filtered.length === 0 && !canCreate ? (
						<p className="px-2 py-1.5 text-sm text-muted-foreground">
							Aucun label
						</p>
					) : null}
				</div>

				{canCreate ? (
					<Button
						type="button"
						variant="ghost"
						size="sm"
						disabled={isCreating}
						className="mt-1 w-full justify-start gap-2"
						onClick={() => {
							onCreate(query.trim())
							setQuery('')
						}}
					>
						<Plus className="size-4" />
						Créer « {query.trim()} »
					</Button>
				) : null}
			</PopoverContent>
		</Popover>
	)
}
