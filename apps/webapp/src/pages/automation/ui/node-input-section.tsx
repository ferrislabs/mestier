import { useState } from 'react'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type {
	ExecutionInputView,
	NodeInputView,
} from '#/pages/automation/lib/node-view'
import type { UpstreamEntry } from '#/pages/automation/lib/upstream-entries'
import { DataViewer } from '#/pages/automation/ui/data-viewer'
import { NdvEmptyState, NdvPanel } from '#/pages/automation/ui/ndv-panel'
import { NodeInputContent } from '#/pages/automation/ui/node-input-panel'
import {
	type InputPanelMode,
	ItemPager,
	type PassSelector,
	type TestRun,
	VariablesPanel,
} from '#/pages/automation/ui/variables-panel'

function ExecutionInputContent({
	view,
	running,
}: {
	view: ExecutionInputView
	running: boolean
}) {
	switch (view.state) {
		case 'absent':
			return running ? (
				<NdvEmptyState
					title="En attente du test"
					text="Les données de ce nœud s’affichent dès que le test l’atteint."
				/>
			) : (
				<NdvEmptyState
					title="Ce nœud est hors du chemin du test"
					text="Aucun déclencheur n’y mène : le test ne le parcourt pas."
				/>
			)
		case 'not-reached':
			return (
				<NdvEmptyState
					title="Ce nœud n’a pas été atteint"
					text="Le test s’est arrêté ou a suivi une autre branche avant ce nœud."
				/>
			)
		case 'ready':
			return view.items.length === 0 ? (
				<NdvEmptyState
					title="Aucune donnée en entrée"
					text="Ce nœud n’a rien reçu lors du test."
				/>
			) : (
				<div className="grid gap-3">
					<p className="text-muted-foreground text-sm">
						Données reçues par ce nœud lors du test.
					</p>
					<DataViewer items={view.items} />
				</div>
			)
	}
}

function ExecutionInput({
	view,
	running,
	passes,
}: {
	view: ExecutionInputView
	running: boolean
	passes: PassSelector | null
}) {
	return (
		<div className="grid gap-3">
			{passes && <ItemPager noun="Passage" {...passes} />}
			<ExecutionInputContent view={view} running={running} />
		</div>
	)
}

export function NodeInputSection({
	entries,
	fallback,
	execution = null,
	passes = null,
	testRun,
}: {
	entries: readonly UpstreamEntry[]
	fallback: NodeInputView
	execution?: ExecutionInputView | null
	passes?: PassSelector | null
	testRun: TestRun | null
}) {
	const [view, setView] = useState<InputPanelMode>('variables')

	return (
		<NdvPanel title="Entrée" tone="side">
			{entries.length === 0 ? (
				<NodeInputContent view={fallback} />
			) : (
				<div className="grid min-w-0 gap-3 [&>*]:min-w-0">
					<ToggleGroup
						type="single"
						variant="outline"
						size="sm"
						value={view}
						onValueChange={(next) => next && setView(next as InputPanelMode)}
						aria-label="Vue de l’entrée"
						className="justify-self-start"
					>
						<ToggleGroupItem value="variables">Variables</ToggleGroupItem>
						<ToggleGroupItem value="execution">Exécution</ToggleGroupItem>
					</ToggleGroup>
					{view === 'execution' && execution ? (
						<ExecutionInput
							view={execution}
							running={testRun?.running ?? false}
							passes={passes}
						/>
					) : (
						<VariablesPanel
							upstream={entries}
							mode={view}
							testRun={testRun ?? undefined}
						/>
					)}
				</div>
			)}
		</NdvPanel>
	)
}
