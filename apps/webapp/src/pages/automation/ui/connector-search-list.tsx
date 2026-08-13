import { Search } from 'lucide-react'
import { useState } from 'react'
import { cn } from '#/lib/utils'
import { badgeColorFor } from '#/pages/automation/lib/connector-badge'
import type { PaletteFamily } from '#/pages/automation/ui/workflow-editor-ui'

export interface ConnectorSearchListProps {
	families: PaletteFamily[]
	onSelect: (kind: string) => void
}

/**
 * A searchable, family-grouped list of connectors — the one place a
 * connector is picked from anywhere on the canvas: the palette's sidebar
 * wraps it as-is, a node's "+" button and a right-click on the empty pane
 * drop it into a `Popover`/`DropdownMenu` respectively. No content is
 * duplicated across those three call sites, only the wrapper differs.
 *
 * Plain content, not a `Popover` itself: whichever floating container
 * hosts it owns open/close and positioning — this only ever needs
 * `families` and a callback.
 */
export function ConnectorSearchList({
	families,
	onSelect,
}: ConnectorSearchListProps) {
	const [search, setSearch] = useState('')
	const normalized = search.trim().toLowerCase()

	const filtered =
		normalized === ''
			? families
			: families
					.map((group) => ({
						family: group.family,
						connectors: group.connectors.filter(
							(connector) =>
								connector.label.toLowerCase().includes(normalized) ||
								group.family.toLowerCase().includes(normalized),
						),
					}))
					.filter((group) => group.connectors.length > 0)

	return (
		<div className="flex w-64 flex-col" data-testid="connector-search-list">
			<div className="flex items-center gap-2 border-b px-3 py-2">
				<Search className="size-4 shrink-0 text-muted-foreground" />
				<input
					// biome-ignore lint/a11y/noAutofocus: this list only ever renders inside a floating container (a Popover or a DropdownMenu) that just opened from an explicit user action — search is the one thing to do in it, same as a command palette.
					autoFocus
					value={search}
					onChange={(event) => setSearch(event.target.value)}
					// A search input living inside a Radix menu/popover would
					// otherwise have its keystrokes captured by the menu's own
					// roving-focus and type-ahead handling (arrow keys, Home/End,
					// letter keys jumping between items) — this input is not an
					// item, it drives what the items even are.
					onKeyDown={(event) => event.stopPropagation()}
					placeholder="Rechercher un connecteur…"
					className="w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
				/>
			</div>
			<div className="max-h-72 overflow-y-auto p-2">
				{filtered.length === 0 ? (
					<p className="px-2 py-3 text-sm text-muted-foreground">
						Aucun connecteur ne correspond
					</p>
				) : (
					filtered.map((group) => (
						<div key={group.family} className="mb-2 last:mb-0">
							<p className="px-2 pb-1 text-xs font-semibold uppercase text-muted-foreground">
								{group.family}
							</p>
							{group.connectors.map((connector) => (
								<button
									key={connector.kind}
									type="button"
									className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-muted"
									onClick={() => onSelect(connector.kind)}
								>
									<span
										className={cn(
											'flex size-5 shrink-0 items-center justify-center rounded-md text-[10px] font-semibold',
											badgeColorFor(group.family),
										)}
									>
										{group.family.charAt(0).toUpperCase()}
									</span>
									<span className="truncate">{connector.label}</span>
								</button>
							))}
						</div>
					))
				)}
			</div>
		</div>
	)
}
