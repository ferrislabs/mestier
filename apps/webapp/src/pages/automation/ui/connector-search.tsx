import { Search } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Input } from '#/components/ui/input'
import { connectorAppearance } from '#/pages/automation/lib/connector-appearance'
import { searchConnectors } from '#/pages/automation/lib/connector-search'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'

export interface ConnectorSearchProps {
	connectors: Schemas.ConnectorDescriptorResponse[]
	onSelect: (connector: Schemas.ConnectorDescriptorResponse) => void
}

const ROW =
	'flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring'

export function ConnectorSearch({
	connectors,
	onSelect,
}: ConnectorSearchProps) {
	const [query, setQuery] = useState('')
	const groups = searchConnectors(connectors, query)

	return (
		<div className="flex w-[320px] max-w-full flex-col bg-card">
			<header className="border-border border-b bg-muted/50 px-4 py-3">
				<h2 className="font-semibold text-base leading-tight">
					Que doit-il se passer ensuite ?
				</h2>
			</header>
			<div className="relative px-3 py-3">
				<Search
					className="pointer-events-none absolute top-1/2 left-[22px] size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					autoFocus
					placeholder="Rechercher un connecteur…"
					aria-label="Rechercher un connecteur"
					value={query}
					onChange={(event) => setQuery(event.target.value)}
					className="pl-8"
				/>
			</div>
			<div className="max-h-80 overflow-y-auto px-2 pb-3">
				{groups.length === 0 ? (
					<p className="px-2 py-8 text-center text-muted-foreground text-sm">
						Aucun connecteur
					</p>
				) : (
					groups.map((group) => {
						const appearance = connectorAppearance(group.family)
						return (
							<section
								key={group.family}
								aria-label={group.family}
								className="mb-2"
							>
								<h3 className="px-3 pt-2 pb-1 font-medium text-muted-foreground text-xs">
									{group.family}
								</h3>
								<ul>
									{group.connectors.map((connector) => (
										<li key={connector.kind}>
											<button
												type="button"
												className={ROW}
												onClick={() => onSelect(connector)}
											>
												<KindIconTile
													icon={appearance.icon}
													hue={appearance.hue}
													size="sm"
												/>
												<span className="min-w-0 flex-1 truncate font-medium text-sm">
													{connector.label}
												</span>
											</button>
										</li>
									))}
								</ul>
							</section>
						)
					})
				)}
			</div>
		</div>
	)
}
