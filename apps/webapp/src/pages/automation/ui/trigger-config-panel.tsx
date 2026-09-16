import { TriangleAlert, X } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import { Label } from '#/components/ui/label'
import { Tabs, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { groupEventsByPrefix } from '#/pages/automation/lib/trigger-events'

export interface TriggerConfigPanelProps {
	events: Schemas.EventDescriptorResponse[]
	mode: 'events' | 'manual'
	selectedEventNames: string[]
	isSaving: boolean
	saveError: string | null
	onClose: () => void
	onSave: (mode: 'events' | 'manual', eventNames: string[]) => void
}

export function TriggerConfigPanel({
	events,
	mode: initialMode,
	selectedEventNames,
	isSaving,
	saveError,
	onClose,
	onSave,
}: TriggerConfigPanelProps) {
	const [mode, setMode] = useState(initialMode)
	const [pending, setPending] = useState(() => new Set(selectedEventNames))
	const groups = groupEventsByPrefix(events)

	function toggle(name: string, checked: boolean) {
		setPending((current) => {
			const next = new Set(current)
			if (checked) next.add(name)
			else next.delete(name)
			return next
		})
	}

	return (
		<div
			data-testid="trigger-config-panel"
			className="flex min-h-0 w-[360px] flex-col overflow-hidden border-l bg-card"
		>
			<div className="flex items-center justify-between gap-2 border-b px-4 py-3">
				<span className="truncate font-medium">Déclencheur</span>
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
					value={mode}
					onValueChange={(value) => setMode(value as 'events' | 'manual')}
				>
					<TabsList className="w-full">
						<TabsTrigger value="events">Sur événement(s)</TabsTrigger>
						<TabsTrigger value="manual">Manuel</TabsTrigger>
					</TabsList>
				</Tabs>
			</div>

			{mode === 'events' && pending.size === 0 ? (
				<div
					role="alert"
					className="flex items-center gap-2 border-b border-amber-500/30 bg-amber-500/10 px-4 py-2 text-sm text-amber-700"
				>
					<TriangleAlert className="size-4 shrink-0" />
					Aucun événement sélectionné : ce workflow ne se déclenchera jamais.
				</div>
			) : null}

			{saveError ? (
				<div
					role="alert"
					className="border-b border-destructive/30 bg-destructive/10 px-4 py-2 text-sm text-destructive"
				>
					{saveError}
				</div>
			) : null}

			{mode === 'events' ? (
				<div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto overscroll-contain p-4">
					{groups.map((group) => (
						<div key={group.prefix} className="flex flex-col gap-2">
							<p className="text-xs font-medium uppercase text-muted-foreground">
								{group.prefix}
							</p>
							{group.events.map((event) => {
								const id = `trigger-event-${event.name}`
								return (
									<div key={event.name} className="flex items-center gap-2">
										<Checkbox
											id={id}
											checked={pending.has(event.name)}
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
			) : (
				<div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto overscroll-contain p-4 text-sm text-muted-foreground">
					Ce workflow ne se déclenche que lorsqu’on l’exécute manuellement,
					depuis « Exécuter maintenant ».
				</div>
			)}

			<div className="border-t p-4">
				<Button
					className="w-full"
					disabled={isSaving}
					onClick={() => onSave(mode, mode === 'events' ? [...pending] : [])}
				>
					{isSaving ? 'Enregistrement…' : 'Enregistrer'}
				</Button>
			</div>
		</div>
	)
}
