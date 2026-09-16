import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Input } from '#/components/ui/input'
import { searchConnectors } from '#/pages/automation/lib/connector-search'

export interface ConnectorSearchProps {
	connectors: Schemas.ConnectorDescriptorResponse[]
	onSelect: (connector: Schemas.ConnectorDescriptorResponse) => void
}

export function ConnectorSearch({
	connectors,
	onSelect,
}: ConnectorSearchProps) {
	const [query, setQuery] = useState('')
	const groups = searchConnectors(connectors, query)

	return (
		<div className="flex w-64 flex-col gap-2">
			<Input
				autoFocus
				placeholder="Rechercher un connecteur…"
				value={query}
				onChange={(event) => setQuery(event.target.value)}
			/>
			<div className="flex max-h-72 flex-col gap-3 overflow-y-auto">
				{groups.length === 0 ? (
					<p className="px-2 py-4 text-center text-sm text-muted-foreground">
						Aucun connecteur
					</p>
				) : (
					groups.map((group) => (
						<div key={group.family} className="flex flex-col gap-1">
							<p className="px-2 text-xs font-medium text-muted-foreground uppercase">
								{group.family}
							</p>
							{group.connectors.map((connector) => (
								<button
									key={connector.kind}
									type="button"
									className="rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent"
									onClick={() => onSelect(connector)}
								>
									{connector.label}
								</button>
							))}
						</div>
					))
				)}
			</div>
		</div>
	)
}
