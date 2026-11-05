import type { Schemas } from '#/api/api.client'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import {
	eventContractSource,
	hasContract,
} from '#/pages/automation/lib/event-contracts'
import { payloadContractSource } from '#/pages/automation/lib/payload-contract'
import { JsonView } from '#/pages/automation/ui/json-view'
import { TypeScriptView } from '#/pages/automation/ui/typescript-view'

function contractSource(event: Schemas.EventDescriptorResponse): string {
	return hasContract(event.name)
		? eventContractSource(event.name)
		: payloadContractSource(event.name, event.payload_example)
}

export function EventPayloadTabs({
	event,
	className,
}: {
	event: Schemas.EventDescriptorResponse
	className?: string
}) {
	return (
		<Tabs defaultValue="contract" className={className}>
			<TabsList aria-label="Vue du payload">
				<TabsTrigger value="contract">Contrat TypeScript</TabsTrigger>
				<TabsTrigger value="example">Exemple de payload</TabsTrigger>
			</TabsList>
			<TabsContent value="contract" className="mt-2">
				<TypeScriptView source={contractSource(event)} />
			</TabsContent>
			<TabsContent value="example" className="mt-2">
				<JsonView value={event.payload_example} />
			</TabsContent>
		</Tabs>
	)
}
