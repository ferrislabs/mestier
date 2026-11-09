import { ChevronRight } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Badge } from '#/components/ui/badge'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { CursusSpace } from '#/pages/cursus/me/feature/use-my-cursus'

interface PortalSpacesListProps {
	spaces: CursusSpace[]
	isLoading: boolean
	onOpen: (space: CursusSpace) => void
}

export function PortalSpacesList({
	spaces,
	isLoading,
	onOpen,
}: PortalSpacesListProps) {
	return (
		<PageShell className="max-w-3xl">
			<PageHeader
				title="Mes espaces"
				description="Les centres de formation dans lesquels vous suivez un cursus."
			/>
			{isLoading ? (
				<p className="text-muted-foreground">Chargement…</p>
			) : spaces.length === 0 ? (
				<EmptyState
					title="Aucun espace Cursus"
					hint="Pour accéder à un espace, ouvrez le lien d’invitation envoyé par votre centre de formation."
				/>
			) : (
				<ul className="grid gap-2">
					{spaces.map((space) => (
						<li key={space.organization_id}>
							<button
								type="button"
								className="flex w-full items-center justify-between gap-3 rounded-lg border bg-card px-4 py-3 text-left hover:bg-muted"
								onClick={() => onOpen(space)}
							>
								<span className="font-medium">{space.name}</span>
								<span className="flex items-center gap-2">
									{space.is_member ? (
										<Badge variant="outline">Console</Badge>
									) : null}
									<ChevronRight className="size-4 text-muted-foreground" />
								</span>
							</button>
						</li>
					))}
				</ul>
			)}
		</PageShell>
	)
}
