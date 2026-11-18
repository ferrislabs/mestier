import { Search, X } from 'lucide-react'
import type { KeyboardEvent } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'
import type { NodeChoiceGroup } from '#/pages/survey/lib/node-catalogue'
import type { NodeChoice } from '#/pages/survey/types'

export const CREATOR_SEARCH_ID = 'survey-node-creator-search'
const LIST_ID = 'survey-node-creator-list'

const ROW =
	'flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring'

function optionId(choice: NodeChoice): string {
	return `survey-node-choice-${choice}`
}

export function SurveyNodeCreator({
	attachedTo,
	query,
	groups,
	active,
	onQueryChange,
	onKeyDown,
	onPick,
	onClose,
}: {
	attachedTo: string | null
	query: string
	groups: readonly NodeChoiceGroup[]
	active: NodeChoice | null
	onQueryChange: (query: string) => void
	onKeyDown: (event: KeyboardEvent<HTMLElement>) => void
	onPick: (choice: NodeChoice) => void
	onClose: () => void
}) {
	return (
		<Dialog
			open
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		>
			<DialogContent
				showCloseButton={false}
				onOpenAutoFocus={(event) => {
					event.preventDefault()
					document.getElementById(CREATOR_SEARCH_ID)?.focus()
				}}
				onKeyDown={onKeyDown}
				className="flex h-[min(720px,calc(100dvh-5rem))] w-[calc(100vw-2.5rem)] max-w-2xl flex-col gap-0 overflow-hidden p-0 sm:max-w-2xl"
			>
				<header className="flex items-start gap-2 border-border border-b bg-muted/50 px-4 py-3">
					<div className="min-w-0 flex-1 py-1">
						<DialogTitle className="font-semibold text-base leading-tight">
							Ajouter un nœud
						</DialogTitle>
						<DialogDescription className="mt-1 text-xs leading-snug">
							{attachedTo
								? `Il sera relié à la sortie libre de « ${attachedTo} ».`
								: 'Reliez-le ensuite en tirant une liaison depuis une sortie.'}
						</DialogDescription>
					</div>
					<DialogClose asChild>
						<Button
							variant="ghost"
							size="icon-sm"
							aria-label="Fermer le panneau des nœuds"
						>
							<X />
						</Button>
					</DialogClose>
				</header>
				<div className="relative px-4 py-3">
					<Search
						className="pointer-events-none absolute top-1/2 left-[26px] size-4 -translate-y-1/2 text-muted-foreground"
						aria-hidden
					/>
					<Input
						id={CREATOR_SEARCH_ID}
						type="search"
						role="combobox"
						aria-expanded
						aria-controls={LIST_ID}
						aria-activedescendant={active ? optionId(active) : undefined}
						value={query}
						onChange={(event) => onQueryChange(event.target.value)}
						placeholder="Rechercher une question, une condition…"
						aria-label="Rechercher un nœud"
						className="pl-8"
					/>
				</div>
				<div id={LIST_ID} className="min-h-0 flex-1 overflow-y-auto px-3 pb-4">
					{groups.length === 0 ? (
						<p className="px-2 py-8 text-center text-muted-foreground text-sm">
							Aucun nœud ne correspond à « {query.trim()} ».
						</p>
					) : (
						groups.map((group) => (
							<section key={group.id} aria-label={group.label} className="mb-2">
								<h3 className="px-3 pt-2 pb-1 font-medium text-muted-foreground text-xs">
									{group.label}
								</h3>
								<ul className="grid gap-0.5 sm:grid-cols-2">
									{group.entries.map((entry) => (
										<li key={entry.choice}>
											<button
												type="button"
												id={optionId(entry.choice)}
												aria-label={entry.title}
												aria-describedby={`${optionId(entry.choice)}-description`}
												aria-current={entry.choice === active || undefined}
												onClick={() => onPick(entry.choice)}
												className={cn(
													ROW,
													entry.choice === active && 'bg-accent',
												)}
											>
												<KindIconTile
													icon={entry.icon}
													hue={entry.hue}
													size="md"
												/>
												<span className="min-w-0 flex-1">
													<span className="block truncate font-medium text-sm">
														{entry.title}
													</span>
													<span
														id={`${optionId(entry.choice)}-description`}
														className="block text-muted-foreground text-xs leading-snug"
													>
														{entry.description}
													</span>
												</span>
											</button>
										</li>
									))}
								</ul>
							</section>
						))
					)}
				</div>
			</DialogContent>
		</Dialog>
	)
}
