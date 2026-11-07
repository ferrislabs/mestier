import type { OperationOutput } from '#/pages/automation/lib/collection-node'
import { JsonView } from '#/pages/automation/ui/json-view'
import { NdvPanel } from '#/pages/automation/ui/ndv-panel'

function isEmptyBody(body: unknown): boolean {
	return (
		body === null ||
		body === undefined ||
		(typeof body === 'object' &&
			!Array.isArray(body) &&
			Object.keys(body).length === 0)
	)
}

export function OperationOutputPanel({
	summary,
	status,
	output,
}: {
	summary: string
	status: string
	output: OperationOutput['output']
}) {
	return (
		<NdvPanel title="Sortie" tone="side">
			<div className="grid min-w-0 gap-3 [&>*]:min-w-0">
				<p className="text-muted-foreground text-sm">
					Ce nœud n’a pas encore été exécuté. Exemple de réponse {status} de{' '}
					<span className="font-medium text-foreground">{summary}</span>,
					d’après la spec importée.
				</p>
				<JsonView value={output} />
				{isEmptyBody(output.body) && (
					<p className="rounded-md border border-border border-dashed px-3 py-4 text-muted-foreground text-sm">
						La spec ne décrit pas le contenu de cette réponse : le corps attendu
						est vide (
						<code className="font-mono">{JSON.stringify(output.body)}</code>
						).
					</p>
				)}
			</div>
		</NdvPanel>
	)
}
