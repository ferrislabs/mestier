import { type ReactNode, useState } from 'react'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type {
	NodeOutputView,
	OutputBranch,
} from '#/pages/automation/lib/node-view'
import { DataViewer } from '#/pages/automation/ui/data-viewer'
import { ErrorBox } from '#/pages/automation/ui/ndv-error-box'
import { NdvEmptyState, NdvPanel } from '#/pages/automation/ui/ndv-panel'

export function BranchedData({
	branches,
}: {
	branches: readonly OutputBranch[]
}) {
	const [branchId, setBranchId] = useState(branches[0]?.id ?? '')
	const selected =
		branches.find((branch) => branch.id === branchId) ?? branches[0]
	return (
		<div className="grid gap-3">
			{branches.length > 1 && (
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={selected?.id}
					onValueChange={(value) => {
						if (value) setBranchId(value)
					}}
					aria-label="Sortie du nœud"
				>
					{branches.map((branch) => (
						<ToggleGroupItem key={branch.id} value={branch.id}>
							{branch.label} ({branch.items.length})
						</ToggleGroupItem>
					))}
				</ToggleGroup>
			)}
			{selected && <DataViewer key={selected.id} items={selected.items} />}
		</div>
	)
}

function OutputContent({
	view,
	errorAction,
}: {
	view: NodeOutputView
	errorAction?: ReactNode
}) {
	switch (view.state) {
		case 'not-executed':
			return (
				<NdvEmptyState
					title="Ce nœud n’a pas été exécuté"
					text="Testez le workflow pour afficher les données produites par ce nœud."
				/>
			)
		case 'error':
			return (
				<ErrorBox
					message={view.message}
					detail={view.detail}
					action={errorAction}
				/>
			)
		case 'ready':
			return <BranchedData branches={view.branches} />
	}
}

export function NodeOutputPanel({
	view,
	errorAction,
}: {
	view: NodeOutputView
	errorAction?: ReactNode
}) {
	return (
		<NdvPanel title="Sortie" tone="side">
			<OutputContent view={view} errorAction={errorAction} />
		</NdvPanel>
	)
}
