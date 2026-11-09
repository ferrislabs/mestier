import { EmptyState } from '#/components/empty-state'
import { Badge } from '#/components/ui/badge'
import { PageHeader, PageShell } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import type { MyTriad } from '#/pages/cursus/me/feature/use-my-cursus'
import { ROLE_LABELS, readOnlySince } from '#/pages/cursus/me/lib/labels'
import { formatDate } from '#/pages/cursus/shared/lib/format'

interface MyTriadsListProps {
	title: string
	description?: string
	triads: MyTriad[]
	isLoading: boolean
	onOpen: (triadId: string) => void
}

export function MyTriadsList({
	title,
	description,
	triads,
	isLoading,
	onOpen,
}: MyTriadsListProps) {
	return (
		<PageShell>
			<PageHeader title={title} description={description} />
			{isLoading ? (
				<p className="text-muted-foreground">Chargement…</p>
			) : triads.length === 0 ? (
				<EmptyState
					title="Aucun trinôme"
					hint="Vous n’êtes rattaché à aucun trinôme pour le moment."
				/>
			) : (
				<div className="rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow>
								<TableHead>Apprenti</TableHead>
								<TableHead>Promotion</TableHead>
								<TableHead>Mon rôle</TableHead>
								<TableHead>Statut</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{triads.map((triad) => (
								<TableRow
									key={triad.triad_id}
									className="cursor-pointer"
									onClick={() => onOpen(triad.triad_id)}
								>
									<TableCell>{triad.apprentice_name}</TableCell>
									<TableCell>
										{triad.promotion_label} · {triad.training_name}
									</TableCell>
									<TableCell>{ROLE_LABELS[triad.role]}</TableCell>
									<TableCell>
										<StatusCell triad={triad} />
									</TableCell>
								</TableRow>
							))}
						</TableBody>
					</Table>
				</div>
			)}
		</PageShell>
	)
}

function StatusCell({ triad }: { triad: MyTriad }) {
	const since = readOnlySince(triad.access)
	if (since) {
		return (
			<span className="flex flex-col items-start gap-1">
				<Badge variant="secondary">Historique, lecture seule</Badge>
				<span className="text-[12px] text-muted-foreground">
					depuis le {formatDate(since)}
				</span>
			</span>
		)
	}
	return (
		<Badge variant="outline">{triad.closed_at ? 'Clôturé' : 'En cours'}</Badge>
	)
}
