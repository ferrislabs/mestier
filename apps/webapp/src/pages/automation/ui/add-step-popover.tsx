import { type ReactNode, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { TRIGGER_APPEARANCE } from '#/pages/automation/lib/connector-appearance'
import { ConnectorSearch } from '#/pages/automation/ui/connector-search'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'

const ROW =
	'flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring'

const TRIGGERS: readonly {
	label: string
	description: string
	kind: Schemas.TriggerKindDto
}[] = [
	{
		label: 'Déclencheur manuel',
		description: 'Démarre le workflow à la demande',
		kind: 'Manual',
	},
	{
		label: 'Sur événement(s)',
		description: 'Démarre le workflow quand un événement de Mestier survient',
		kind: { Events: [] },
	},
]

export function AddStepPopover({
	catalogue,
	onSelectConnector,
	onSelectTrigger,
	children,
}: {
	catalogue: Schemas.ConnectorDescriptorResponse[]
	onSelectConnector: (connector: Schemas.ConnectorDescriptorResponse) => void
	onSelectTrigger: (kind: Schemas.TriggerKindDto) => void
	children: ReactNode
}) {
	const [open, setOpen] = useState(false)

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>{children}</PopoverTrigger>
			<PopoverContent align="end" className="w-auto p-0">
				<section
					aria-label="Déclencheurs"
					className="border-border border-b p-2"
				>
					<h3 className="px-3 pt-1 pb-1 font-medium text-muted-foreground text-xs">
						Ajouter un déclencheur
					</h3>
					<ul>
						{TRIGGERS.map((trigger) => (
							<li key={trigger.label}>
								<button
									type="button"
									className={ROW}
									onClick={() => {
										onSelectTrigger(trigger.kind)
										setOpen(false)
									}}
								>
									<KindIconTile
										icon={TRIGGER_APPEARANCE.icon}
										hue={TRIGGER_APPEARANCE.hue}
										size="sm"
									/>
									<span className="min-w-0 flex-1">
										<span className="block truncate font-medium text-sm">
											{trigger.label}
										</span>
										<span className="block text-muted-foreground text-xs leading-snug">
											{trigger.description}
										</span>
									</span>
								</button>
							</li>
						))}
					</ul>
				</section>
				<ConnectorSearch
					connectors={catalogue}
					onSelect={(connector) => {
						onSelectConnector(connector)
						setOpen(false)
					}}
				/>
			</PopoverContent>
		</Popover>
	)
}
