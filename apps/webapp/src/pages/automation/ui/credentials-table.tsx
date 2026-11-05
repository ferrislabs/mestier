import { KeyRound } from 'lucide-react'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { credentialAppearance } from '#/pages/automation/lib/credential-appearance'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'

export interface CredentialRow {
	id: string
	name: string
	kind: string
	kindLabel: string
	origin: 'supplied' | 'generated'
	updatedAt: string
}

export function CredentialsTable({
	credentials,
	onOpen,
}: {
	credentials: readonly CredentialRow[]
	onOpen: ((row: CredentialRow) => void) | null
}) {
	if (credentials.length === 0)
		return (
			<div className="flex flex-col items-center rounded-lg border border-border border-dashed bg-card px-6 py-14 text-center">
				<KeyRound className="mb-3 size-8 text-muted-foreground" aria-hidden />
				<p className="font-medium text-base">Aucune identification</p>
				<p className="mt-1 max-w-sm text-muted-foreground text-sm">
					Ajoutez les accès dont vos workflows ont besoin pour parler à d’autres
					systèmes.
				</p>
			</div>
		)
	return (
		<div className="overflow-hidden rounded-lg border border-border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="px-4 text-muted-foreground">
							Identification
						</TableHead>
						<TableHead className="px-4 text-muted-foreground">Type</TableHead>
						<TableHead className="px-4 text-muted-foreground">
							Origine
						</TableHead>
						<TableHead className="px-4 text-muted-foreground">
							Modifiée le
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{credentials.map((credential) => {
						const appearance = credentialAppearance(credential.kind)
						return (
							<TableRow
								key={credential.id}
								className={onOpen ? 'cursor-pointer' : undefined}
								onClick={onOpen ? () => onOpen(credential) : undefined}
							>
								<TableCell className="max-w-0 px-4 py-3">
									<span className="flex min-w-0 items-center gap-3">
										<KindIconTile
											icon={appearance.icon}
											hue={appearance.hue}
											size="sm"
										/>
										<span className="min-w-0">
											{onOpen ? (
												<button
													type="button"
													onClick={(event) => {
														event.stopPropagation()
														onOpen(credential)
													}}
													className="block max-w-full truncate rounded-sm text-left font-medium text-base outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
												>
													{credential.name}
												</button>
											) : (
												<span className="block max-w-full truncate font-medium text-base">
													{credential.name}
												</span>
											)}
										</span>
									</span>
								</TableCell>
								<TableCell className="px-4">
									<span className="block text-sm">{credential.kindLabel}</span>
								</TableCell>
								<TableCell className="px-4 text-muted-foreground">
									{credential.origin === 'generated'
										? 'Générée par Mestier'
										: 'Fournie'}
								</TableCell>
								<TableCell className="px-4 text-muted-foreground">
									{new Date(credential.updatedAt).toLocaleDateString('fr-FR', {
										day: 'numeric',
										month: 'short',
										year: 'numeric',
									})}
								</TableCell>
							</TableRow>
						)
					})}
				</TableBody>
			</Table>
		</div>
	)
}
