import { Link } from '@tanstack/react-router'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { buildOrgPath } from '#/modules/org-path'
import { dafNumberLabel, formatDate } from '#/pages/daf/lib/daf-status'
import { DafStatusBadge } from '#/pages/daf/ui/daf-status-badge'

export interface DafSummaryRow {
	id: string
	number: number
	designation: string
	price_ref?: string | null
	status: Parameters<typeof DafStatusBadge>[0]['status']
	sent_on?: string | null
	issued_on: string
	returned_on?: string | null
	review_note?: string | null
}

export function DafSummaryTable({
	organizationSlug,
	projectId,
	rows,
}: {
	organizationSlug: string
	projectId: string
	rows: readonly DafSummaryRow[]
}) {
	return (
		<Table>
			<TableHeader>
				<TableRow>
					<TableHead>N°</TableHead>
					<TableHead>N° prix</TableHead>
					<TableHead>Désignation</TableHead>
					<TableHead>Envoyé</TableHead>
					<TableHead>Diffusion</TableHead>
					<TableHead>Retour</TableHead>
					<TableHead>Statut</TableHead>
					<TableHead>Observation</TableHead>
				</TableRow>
			</TableHeader>
			<TableBody>
				{rows.map((row) => (
					<TableRow key={row.id}>
						<TableCell className="whitespace-nowrap font-medium">
							<Link
								to={buildOrgPath(
									organizationSlug,
									'/projet/$projectId/daf/$dafId',
								)}
								params={{ projectId, dafId: row.id }}
								className="rounded-sm outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
							>
								{dafNumberLabel(row.number)}
							</Link>
						</TableCell>
						<TableCell>{row.price_ref || '–'}</TableCell>
						<TableCell>{row.designation}</TableCell>
						<TableCell>{formatDate(row.sent_on)}</TableCell>
						<TableCell>{formatDate(row.issued_on)}</TableCell>
						<TableCell>{formatDate(row.returned_on)}</TableCell>
						<TableCell>
							<DafStatusBadge status={row.status} />
						</TableCell>
						<TableCell className="max-w-64 truncate text-muted-foreground">
							{row.review_note || '–'}
						</TableCell>
					</TableRow>
				))}
			</TableBody>
		</Table>
	)
}
