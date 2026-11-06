import { SCHEDULE_PAYLOAD_KEYS } from '#/pages/automation/lib/schedule'
import { NdvPanel } from '#/pages/automation/ui/ndv-panel'
import { OperationContractTabs } from '#/pages/automation/ui/operation-contract-tabs'

export function ScheduleInputPanel({
	contract,
	example,
}: {
	contract: string
	example: unknown
}) {
	return (
		<NdvPanel title="Entrée" tone="side">
			<div className="grid min-w-0 gap-3 [&>*]:min-w-0">
				<p className="text-muted-foreground text-sm">
					Le workflow démarre à l’heure prévue. Les nœuds suivants reçoivent ce
					payload.
				</p>
				<dl className="grid gap-1.5 text-xs">
					{SCHEDULE_PAYLOAD_KEYS.map((entry) => (
						<div key={entry.key} className="grid gap-0.5">
							<dt className="font-mono font-semibold">{entry.key}</dt>
							<dd className="text-muted-foreground">{entry.description}</dd>
						</div>
					))}
				</dl>
				<OperationContractTabs
					source={contract}
					example={example}
					exampleLabel="Exemple de payload"
				/>
			</div>
		</NdvPanel>
	)
}
