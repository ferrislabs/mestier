import type { SupportToolbarProps } from '#/pages/support/types'
import { TicketFiltersPopover } from '#/pages/support/ui/ticket-filters-popover'
import {
	DisplayFieldsMenu,
	TicketSearchInput,
	ViewToggle,
} from '#/pages/support/ui/tickets-toolbar'

export function TicketListHeader(props: SupportToolbarProps) {
	return (
		<header className="grid gap-3 border-border border-b px-4 py-3">
			<div className="flex items-center gap-2">
				<h1 className="truncate font-semibold text-base">{props.title}</h1>
				<span
					className="rounded-full bg-muted px-2 py-px text-muted-foreground text-xs tabular-nums"
					title="Tickets affichés, triés par priorité puis par activité"
				>
					{props.count}
				</span>
				<span className="ml-auto flex items-center gap-1.5">
					<DisplayFieldsMenu
						iconOnly
						options={props.displayFieldOptions}
						displayFields={props.displayFields}
						canEdit={props.canEditDisplayFields}
						onChange={props.onDisplayFieldsChange}
					/>
					<ViewToggle
						iconOnly
						view={props.view}
						onViewChange={props.onViewChange}
					/>
				</span>
			</div>
			<div className="flex items-center gap-2">
				<TicketSearchInput
					query={props.query}
					onQueryChange={props.onQueryChange}
					className="min-w-0 flex-1"
				/>
				<TicketFiltersPopover {...props} />
			</div>
		</header>
	)
}
