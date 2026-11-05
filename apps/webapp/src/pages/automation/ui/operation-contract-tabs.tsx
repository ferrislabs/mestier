import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { JsonView } from '#/pages/automation/ui/json-view'
import { TypeScriptView } from '#/pages/automation/ui/typescript-view'

export function OperationContractTabs({
	source,
	example,
	exampleLabel = 'Exemple de réponse',
	className,
}: {
	source: string
	example: unknown
	exampleLabel?: string
	className?: string
}) {
	return (
		<Tabs defaultValue="contract" className={className}>
			<TabsList aria-label="Vue du contrat">
				<TabsTrigger value="contract">Contrat TypeScript</TabsTrigger>
				<TabsTrigger value="example">{exampleLabel}</TabsTrigger>
			</TabsList>
			<TabsContent value="contract" className="mt-2">
				<TypeScriptView source={source} />
			</TabsContent>
			<TabsContent value="example" className="mt-2">
				{example === null || example === undefined ? (
					<p className="rounded-md border border-border border-dashed px-3 py-4 text-muted-foreground text-sm">
						Cette opération ne renvoie pas de corps JSON.
					</p>
				) : (
					<JsonView value={example} />
				)}
			</TabsContent>
		</Tabs>
	)
}
