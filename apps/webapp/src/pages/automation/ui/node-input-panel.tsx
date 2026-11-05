import type { NodeInputView } from '#/pages/automation/lib/node-view'
import { DataViewer } from '#/pages/automation/ui/data-viewer'
import { NdvEmptyState, NdvPanel } from '#/pages/automation/ui/ndv-panel'

export function NodeInputContent({ view }: { view: NodeInputView }) {
	switch (view.state) {
		case 'trigger':
			return (
				<NdvEmptyState
					title="Aucune donnée en entrée"
					text="Ce nœud est un déclencheur : il démarre le workflow sans recevoir de données."
				/>
			)
		case 'unconnected':
			return (
				<NdvEmptyState
					title="Nœud non connecté"
					text="Reliez ce nœud à un nœud précédent pour récupérer ses données."
				/>
			)
		case 'not-executed':
			return (
				<NdvEmptyState
					title="Aucune donnée disponible"
					text={`Le nœud précédent (${view.sourceLabels.join(', ')}) n’a pas produit de données lors de la dernière exécution.`}
				/>
			)
		case 'ready':
			return (
				<div className="grid gap-3">
					<p className="text-muted-foreground text-sm">
						Depuis{' '}
						<span className="font-medium text-foreground">
							{view.sourceLabels.join(', ')}
						</span>
					</p>
					<DataViewer items={view.items} />
				</div>
			)
	}
}

export function NodeInputPanel({ view }: { view: NodeInputView }) {
	return (
		<NdvPanel title="Entrée" tone="side">
			<NodeInputContent view={view} />
		</NdvPanel>
	)
}
