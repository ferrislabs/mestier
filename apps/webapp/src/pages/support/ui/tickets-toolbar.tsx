import { Columns3, List, Search, SlidersHorizontal } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuCheckboxItem,
	DropdownMenuContent,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { cn } from '#/lib/utils'
import { toggleDisplayField } from '#/pages/support/lib/display-fields'
import type { SupportToolbarProps, SupportView } from '#/pages/support/types'
import { TicketFiltersPopover } from '#/pages/support/ui/ticket-filters-popover'

export function TicketSearchInput({
	query,
	onQueryChange,
	className,
}: {
	query: string
	onQueryChange: (query: string) => void
	className?: string
}) {
	return (
		<div className={cn('relative', className)}>
			<Search
				className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
				aria-hidden
			/>
			<Input
				type="search"
				value={query}
				onChange={(event) => onQueryChange(event.target.value)}
				placeholder="N°, sujet, client…"
				aria-label="Rechercher un ticket"
				className="pl-8"
			/>
		</div>
	)
}

export function DisplayFieldsMenu({
	options,
	displayFields,
	canEdit,
	onChange,
	iconOnly = false,
}: {
	options: readonly string[]
	displayFields: readonly string[]
	canEdit: boolean
	onChange: (fields: string[]) => void
	iconOnly?: boolean
}) {
	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button
					variant="outline"
					size={iconOnly ? 'icon' : 'default'}
					aria-label="Champs affichés"
					title="Champs affichés"
				>
					<SlidersHorizontal />
					{!iconOnly && 'Champs affichés'}
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end" className="w-60">
				<DropdownMenuLabel>Champs personnalisés</DropdownMenuLabel>
				<DropdownMenuSeparator />
				{options.length === 0 ? (
					<p className="px-2 py-1.5 text-muted-foreground text-sm">
						Aucun champ transmis pour le moment.
					</p>
				) : (
					options.map((key) => (
						<DropdownMenuCheckboxItem
							key={key}
							checked={displayFields.includes(key)}
							disabled={!canEdit}
							onSelect={(event) => event.preventDefault()}
							onCheckedChange={(checked) =>
								onChange(
									toggleDisplayField(displayFields, options, key, checked),
								)
							}
						>
							{key}
						</DropdownMenuCheckboxItem>
					))
				)}
			</DropdownMenuContent>
		</DropdownMenu>
	)
}

export function ViewToggle({
	view,
	onViewChange,
	iconOnly = false,
}: {
	view: SupportView
	onViewChange: (view: SupportView) => void
	iconOnly?: boolean
}) {
	return (
		<ToggleGroup
			type="single"
			variant="outline"
			value={view}
			onValueChange={(value) => {
				if (value === 'list' || value === 'board') onViewChange(value)
			}}
			aria-label="Affichage"
		>
			<ToggleGroupItem
				value="list"
				aria-label="Liste"
				title="Liste"
				className="px-3"
			>
				<List />
				{!iconOnly && 'Liste'}
			</ToggleGroupItem>
			<ToggleGroupItem
				value="board"
				aria-label="Kanban"
				title="Kanban"
				className="px-3"
			>
				<Columns3 />
				{!iconOnly && 'Kanban'}
			</ToggleGroupItem>
		</ToggleGroup>
	)
}

export function TicketsToolbar(props: SupportToolbarProps) {
	return (
		<div className="mb-4 flex flex-wrap items-center gap-2">
			<TicketSearchInput
				query={props.query}
				onQueryChange={props.onQueryChange}
				className="min-w-[220px] flex-1 sm:max-w-xs"
			/>
			<TicketFiltersPopover {...props} />
			<div className="ml-auto flex items-center gap-2">
				<DisplayFieldsMenu
					options={props.displayFieldOptions}
					displayFields={props.displayFields}
					canEdit={props.canEditDisplayFields}
					onChange={props.onDisplayFieldsChange}
				/>
				<ViewToggle view={props.view} onViewChange={props.onViewChange} />
			</div>
		</div>
	)
}
