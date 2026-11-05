import { useState } from 'react'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type { NodeInputView } from '#/pages/automation/lib/node-view'
import type { UpstreamEntry } from '#/pages/automation/lib/upstream-entries'
import { NdvPanel } from '#/pages/automation/ui/ndv-panel'
import { NodeInputContent } from '#/pages/automation/ui/node-input-panel'
import {
	type InputPanelMode,
	type TestRun,
	VariablesPanel,
} from '#/pages/automation/ui/variables-panel'

export function NodeInputSection({
	entries,
	fallback,
	testRun,
}: {
	entries: readonly UpstreamEntry[]
	fallback: NodeInputView
	testRun: TestRun
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
					<VariablesPanel upstream={entries} mode={view} testRun={testRun} />
				</div>
			)}
		</NdvPanel>
	)
}
