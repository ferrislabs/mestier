import { NdvPanel } from '#/pages/automation/ui/ndv-panel'
import { OperationContractTabs } from '#/pages/automation/ui/operation-contract-tabs'

export function WebhookInputPanel({
	method,
	url,
	contract,
	example,
}: {
	method: string
	url: string
	contract: string
	example: unknown
}) {
	return (
		<NdvPanel title="Entrée" caption="Requête attendue" tone="side">
			<div className="grid min-w-0 gap-3 [&>*]:min-w-0">
				<p className="text-muted-foreground text-sm">
					Le workflow démarre quand un service appelle cette adresse. Les nœuds
					suivants reçoivent le corps de la requête sous cette forme.
				</p>
				<p className="truncate rounded-md border border-border bg-card px-3 py-2 font-mono text-xs">
					<span className="font-semibold">{method}</span> {url}
				</p>
				<OperationContractTabs
					source={contract}
					example={example}
					exampleLabel="Exemple de requête"
				/>
			</div>
		</NdvPanel>
	)
}
