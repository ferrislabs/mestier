import { TriangleAlert, X } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import { Label } from '#/components/ui/label'
import { Tabs, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { groupEventsByPrefix } from '#/pages/automation/lib/trigger-events'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import {
	isManualTrigger,
	triggerEventNames,
} from '#/pages/automation/ui/trigger-node'

export interface TriggerConfigPanelProps {
	trigger: Schemas.PlacedTriggerDto
	events: Schemas.EventDescriptorResponse[]
	errors: ConnectorValidationError[]
	onClose: () => void
	onChange: (kind: Schemas.TriggerKindDto) => void
}

export function TriggerConfigPanel({
	trigger,
	events,
	errors,
	onClose,
	onChange,
}: TriggerConfigPanelProps) {
	const manual = isManualTrigger(trigger)
	const selected = triggerEventNames(trigger)
	const groups = groupEventsByPrefix(events)

	function toggle(name: string, checked: boolean) {
		const next = new Set(selected)
		if (checked) next.add(name)
		else next.delete(name)
		onChange({ Events: [...next] })
	}

	return (
		<div
			data-testid="trigger-config-panel"
			className="flex min-h-0 w-[360px] flex-col overflow-hidden border-l bg-card"
		>
			<div className="flex items-center justify-between gap-2 border-b px-4 py-3">
				<span className="truncate font-medium">Déclencheur {trigger.id}</span>
				<Button
					variant="ghost"
					size="icon-sm"
					aria-label="Fermer"
					onClick={onClose}
				>
					<X className="size-4" />
				</Button>
			</div>

			<div className="border-b px-4 py-3">
				<Tabs
					value={manual ? 'manual' : 'events'}
					onValueChange={(value) =>
						onChange(value === 'manual' ? 'Manual' : { Events: selected })
					}
				>
					<TabsList className="w-full">
						<TabsTrigger value="events">Sur événement(s)</TabsTrigger>
						<TabsTrigger value="manual">Manuel</TabsTrigger>
					</TabsList>
				</Tabs>
			</div>

			{!manual && selected.length === 0 ? (
				<div
					role="alert"
					className="flex items-center gap-2 border-b border-amber-500/30 bg-amber-500/10 px-4 py-2 text-sm text-amber-700"
				>
					<TriangleAlert className="size-4 shrink-0" />
					Aucun événement sélectionné : ce déclencheur ne partira jamais.
				</div>
			) : null}

			{errors.length > 0 ? (
				<div
					role="alert"
					className="border-b border-destructive/30 bg-destructive/10 px-4 py-2 text-sm text-destructive"
				>
					{errors.map((error) => (
						<p key={error.message}>{error.message}</p>
					))}
				</div>
			) : null}

			{manual ? (
				<div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto overscroll-contain p-4 text-sm text-muted-foreground">
					Ce déclencheur ne part que lorsqu’on exécute le workflow à la main,
					depuis « Exécuter maintenant ».
				</div>
			) : (
				<div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto overscroll-contain p-4">
					{groups.map((group) => (
						<div key={group.prefix} className="flex flex-col gap-2">
							<p className="text-xs font-medium uppercase text-muted-foreground">
								{group.prefix}
							</p>
							{group.events.map((event) => {
								const id = `trigger-${trigger.id}-event-${event.name}`
								return (
									<div key={event.name} className="flex items-center gap-2">
										<Checkbox
											id={id}
											checked={selected.includes(event.name)}
											onCheckedChange={(checked) =>
												toggle(event.name, checked === true)
											}
										/>
										<Label htmlFor={id}>{event.label}</Label>
									</div>
								)
							})}
						</div>
					))}
				</div>
			)}
		</div>
	)
}
