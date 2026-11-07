import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { buildOrgPath } from '#/modules/org-path'
import { formatDate } from '#/pages/customers/lib/format'
import { DetailCard, EmptyNote } from '#/pages/customers/ui/detail-card'
import { Money } from '#/pages/customers/ui/money'
import type { DealStage } from '#/pages/deals/lib/deal-types'
import { DealStageBadge } from '#/pages/deals/ui/deal-badges'

export type OpportunityRow = {
	id: string
	title: string
	stage: DealStage
	amount: number
	date: string
}

export function OpportunitiesCard({
	organizationSlug,
	rows,
	action,
}: {
	organizationSlug: string
	rows: readonly OpportunityRow[]
	action?: ReactNode
}) {
	return (
		<DetailCard title="Opportunités" hint="Montants HT" action={action}>
			{rows.length === 0 ? (
				<EmptyNote>Aucune opportunité pour ce client.</EmptyNote>
			) : (
				<ul className="-my-1 divide-y">
					{rows.map((row) => (
						<li key={row.id} className="flex items-center gap-3 py-2.5">
							<div className="min-w-0 flex-1 truncate font-medium text-[13px]">
								<Link
									to={buildOrgPath(organizationSlug, '/crm/pipeline')}
									className="rounded-sm outline-none transition-colors hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring"
								>
									{row.title}
								</Link>
							</div>
							<span className="shrink-0 text-[12px] text-muted-foreground tabular-nums max-sm:hidden">
								{formatDate(row.date)}
							</span>
							<DealStageBadge stage={row.stage} />
							<div className="w-28 shrink-0 text-right text-[13px]">
								<Money cents={row.amount} />
							</div>
						</li>
					))}
				</ul>
			)}
		</DetailCard>
	)
}
