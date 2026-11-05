import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { InvoiceRow, QuoteRow } from '#/pages/customers/lib/activity'
import { formatDate } from '#/pages/customers/lib/format'
import { TAB_INFO } from '#/pages/customers/lib/rubriques'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { DetailCard, EmptyNote } from '#/pages/customers/ui/detail-card'
import { Money } from '#/pages/customers/ui/money'

const LINK =
	'rounded-sm outline-none transition-colors hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring'

function Tile({
	label,
	children,
	tone,
}: {
	label: string
	children: ReactNode
	tone?: 'alert'
}) {
	return (
		<div className="rounded-lg border bg-card px-4 py-3">
			<p className="text-muted-foreground text-sm">{label}</p>
			<p
				className={cn(
					'mt-1 font-semibold text-xl tabular-nums',
					tone === 'alert' && 'text-state-alert',
				)}
			>
				{children}
			</p>
		</div>
	)
}

function Row({
	primary,
	secondary,
	date,
	badge,
	amount,
}: {
	primary: ReactNode
	secondary?: ReactNode
	date?: string
	badge: ReactNode
	amount: ReactNode
}) {
	return (
		<li className="flex items-center gap-3 py-2.5">
			<div className="min-w-0 flex-1">
				<div className="truncate font-medium text-[13px]">{primary}</div>
				{secondary && (
					<div className="truncate text-[12px] text-muted-foreground">
						{secondary}
					</div>
				)}
			</div>
			{date && (
				<span className="shrink-0 text-[12px] text-muted-foreground tabular-nums max-sm:hidden">
					{date}
				</span>
			)}
			<div className="shrink-0">{badge}</div>
			<div className="w-28 shrink-0 text-right text-[13px]">{amount}</div>
		</li>
	)
}

function List({ children }: { children: ReactNode }) {
	return <ul className="-my-1 divide-y">{children}</ul>
}

export function ActivitySection({
	organizationSlug,
	outstanding,
	oldestDueAt,
	late,
	quotes,
	invoices,
	isLoading,
}: {
	organizationSlug: string
	outstanding: number
	oldestDueAt: string | null
	late: boolean
	quotes: readonly QuoteRow[]
	invoices: readonly InvoiceRow[]
	isLoading: boolean
}) {
	return (
		<RubriqueLayout
			title={TAB_INFO.activite.label}
			description={TAB_INFO.activite.description}
			wide
		>
			<div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
				<Tile label="Encours">
					<Money cents={outstanding} />
				</Tile>
				<Tile label="Plus ancienne échéance" tone={late ? 'alert' : undefined}>
					{oldestDueAt ? formatDate(oldestDueAt) : '—'}
				</Tile>
				<Tile label="Devis">{quotes.length}</Tile>
				<Tile label="Factures">{invoices.length}</Tile>
			</div>

			<DetailCard title="Devis" hint="Montants TTC">
				{isLoading ? (
					<EmptyNote>Chargement…</EmptyNote>
				) : quotes.length === 0 ? (
					<EmptyNote>Aucun devis pour ce client.</EmptyNote>
				) : (
					<List>
						{quotes.map((row) => (
							<Row
								key={row.id}
								primary={
									<Link
										to={buildOrgPath(organizationSlug, '/crm/quotes/$quoteId')}
										params={{ quoteId: row.id }}
										className={LINK}
									>
										{row.label}
									</Link>
								}
								secondary={row.title}
								date={formatDate(row.date)}
								badge={<StatusBadge tone={row.tone}>{row.status}</StatusBadge>}
								amount={<Money cents={row.total} />}
							/>
						))}
					</List>
				)}
			</DetailCard>

			<DetailCard title="Factures" hint="Montants TTC">
				{isLoading ? (
					<EmptyNote>Chargement…</EmptyNote>
				) : invoices.length === 0 ? (
					<EmptyNote>Aucune facture pour ce client.</EmptyNote>
				) : (
					<List>
						{invoices.map((row) => (
							<Row
								key={row.id}
								primary={
									<Link
										to={buildOrgPath(
											organizationSlug,
											'/crm/invoices/$invoiceId',
										)}
										params={{ invoiceId: row.id }}
										className={LINK}
									>
										{row.label}
									</Link>
								}
								date={formatDate(row.date)}
								badge={<StatusBadge tone={row.tone}>{row.status}</StatusBadge>}
								amount={<Money cents={row.total} />}
							/>
						))}
					</List>
				)}
			</DetailCard>
		</RubriqueLayout>
	)
}
