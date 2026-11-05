import { Search } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'
import type { EventSearchGroup } from '#/pages/automation/lib/event-search'
import { EventPayloadTabs } from '#/pages/automation/ui/event-payload-tabs'

function EventPreview({ event }: { event: Schemas.EventDescriptorResponse }) {
	return (
		<div>
			<h3 className="font-semibold text-lg leading-tight">{event.label}</h3>
			<dl className="mt-3 flex flex-wrap gap-2 text-xs">
				<div className="flex items-center gap-1.5 rounded-md border border-border px-2 py-1">
					<dt className="text-muted-foreground">Nom</dt>
					<dd className="font-mono">{event.name}</dd>
				</div>
				<div className="flex items-center gap-1.5 rounded-md border border-border px-2 py-1">
					<dt className="text-muted-foreground">Version</dt>
					<dd className="font-mono">{event.version}</dd>
				</div>
				<div className="flex items-center gap-1.5 rounded-md border border-border px-2 py-1">
					<dt className="text-muted-foreground">Sujet</dt>
					<dd className="font-mono">{event.subject_kind}</dd>
				</div>
			</dl>
			<p className="mt-4 max-w-prose text-muted-foreground text-sm">
				Ce payload est garanti pour la version {event.version}. Un changement
				incompatible publie une nouvelle version : ce workflow continue de
				recevoir la forme ci-dessous.
			</p>
			<EventPayloadTabs event={event} className="mt-5" />
		</div>
	)
}

export function MestierEventDialog({
	open,
	onOpenChange,
	query,
	onQueryChange,
	groups,
	selected,
	onSelect,
	onConfirm,
	confirmLabel,
}: {
	open: boolean
	onOpenChange: (open: boolean) => void
	query: string
	onQueryChange: (query: string) => void
	groups: readonly EventSearchGroup[]
	selected: Schemas.EventDescriptorResponse | null
	onSelect: (name: string) => void
	onConfirm: () => void
	confirmLabel: string
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent
				showCloseButton={false}
				className="flex h-[min(40rem,85vh)] max-w-4xl flex-col gap-0 p-0 sm:max-w-4xl"
			>
				<DialogHeader className="border-border border-b px-5 py-4">
					<DialogTitle>Choisir l’événement déclencheur</DialogTitle>
					<DialogDescription>
						Le workflow démarre chaque fois que cet événement se produit dans
						Mestier, et reçoit son payload contractuel.
					</DialogDescription>
				</DialogHeader>
				<div className="grid min-h-0 flex-1 grid-cols-[17rem_1fr]">
					<div className="flex min-h-0 flex-col border-border border-r">
						<div className="relative p-3">
							<Search
								className="pointer-events-none absolute top-1/2 left-[22px] size-4 -translate-y-1/2 text-muted-foreground"
								aria-hidden
							/>
							<Input
								type="search"
								value={query}
								onChange={(event) => onQueryChange(event.target.value)}
								placeholder="Rechercher un événement…"
								aria-label="Rechercher un événement"
								className="pl-8"
								autoFocus
							/>
						</div>
						<div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
							{groups.length === 0 ? (
								<p className="px-2 py-6 text-center text-muted-foreground text-sm">
									Aucun événement ne correspond à « {query.trim()} ».
								</p>
							) : (
								groups.map((group) => (
									<section
										key={group.id}
										aria-label={group.label}
										className="mb-1"
									>
										<h3 className="px-2 pt-2 pb-1 font-medium text-muted-foreground text-xs">
											{group.label}
										</h3>
										<ul>
											{group.events.map((event) => {
												const active = selected?.name === event.name
												return (
													<li key={event.name}>
														<button
															type="button"
															aria-pressed={active}
															onClick={() => onSelect(event.name)}
															onDoubleClick={onConfirm}
															className={cn(
																'w-full rounded-md px-2 py-1.5 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring',
																active && 'bg-accent',
															)}
														>
															<span className="block truncate font-medium text-sm">
																{event.label}
															</span>
															<span className="block truncate font-mono text-[11px] text-muted-foreground">
																{event.name}
															</span>
														</button>
													</li>
												)
											})}
										</ul>
									</section>
								))
							)}
						</div>
					</div>
					<div className="min-h-0 overflow-y-auto p-5">
						{selected ? (
							<EventPreview event={selected} />
						) : (
							<p className="text-muted-foreground text-sm">
								Choisissez un événement dans la liste pour voir son payload.
							</p>
						)}
					</div>
				</div>
				<DialogFooter className="border-border border-t px-5 py-3">
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Annuler
					</Button>
					<Button onClick={onConfirm} disabled={!selected}>
						{confirmLabel}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
