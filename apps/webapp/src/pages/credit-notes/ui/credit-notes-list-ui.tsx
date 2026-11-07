import { Link } from '@tanstack/react-router'
import { Search } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { PageHeader, PageShell } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Customer } from '#/hooks/use-customers'
import type { Invoice } from '#/hooks/use-invoices'
import { buildOrgPath } from '#/modules/org-path'
import {
	creditNoteRows,
	creditNotesSummary,
	filterCreditNoteRows,
	kindsLabel,
} from '#/pages/credit-notes/lib/credit-notes-list'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import { DocBadge } from '#/pages/quotes/ui/doc/badges'
import { AccountLink, InvoiceLink } from '#/pages/quotes/ui/doc/doc-links'
import { Money, StatTile } from '#/pages/quotes/ui/doc/primitives'

interface CreditNotesListUIProps {
	creditNotes: Invoice[]
	invoices: Invoice[]
	customers: Customer[]
	query: string
	isLoading: boolean
	error: string | null
	onRetry: () => void
	onQueryChange: (query: string) => void
}

export function CreditNotesListUI({
	creditNotes,
	invoices,
	customers,
	query,
	isLoading,
	error,
	onRetry,
	onQueryChange,
}: CreditNotesListUIProps) {
	const { activeOrganization } = useActiveOrganization()
	const [text, setText] = useState(query)
	const pushed = useRef(query)

	useEffect(() => {
		if (query === pushed.current) return
		pushed.current = query
		setText(query)
	}, [query])

	const change = (next: string) => {
		setText(next)
		pushed.current = next
		onQueryChange(next)
	}

	const all = creditNoteRows(creditNotes, invoices, customers)
	const rows = filterCreditNoteRows(all, text)
	const summary = creditNotesSummary(all)

	return (
		<PageShell className="gap-0">
			<PageHeader
				className="mb-6"
				title="Avoirs"
				description="Un avoir se crée depuis la facture concernée : ouvre une facture émise, puis « Émettre un avoir » ou « Annuler la facture »."
			/>
			{error ? (
				<div
					role="alert"
					className="mb-4 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-sm"
				>
					<p className="font-medium text-state-alert">{error}</p>
					<Button onClick={onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</div>
			) : null}
			<section
				aria-label="Synthèse des avoirs"
				className="mb-4 grid gap-3 sm:grid-cols-2 lg:max-w-xl"
			>
				<StatTile
					label="Avoirs émis"
					value={summary.count}
					hint={kindsLabel(summary)}
				/>
				<StatTile
					label="Total crédité"
					value={<Money cents={summary.totalCents} />}
					hint="TTC, toutes factures confondues"
				/>
			</section>
			<div className="relative mb-3 w-full max-w-xs">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					type="search"
					value={text}
					onChange={(event) => change(event.target.value)}
					placeholder="Numéro, facture, client, motif"
					aria-label="Rechercher un avoir"
					className="h-8 bg-card pl-8"
				/>
			</div>
			{isLoading ? (
				<div className="rounded-lg border bg-card px-6 py-12 text-center text-muted-foreground text-sm">
					Chargement…
				</div>
			) : rows.length === 0 ? (
				<EmptyState
					title={
						all.length === 0
							? 'Aucun avoir pour l’instant'
							: 'Aucun avoir ne correspond'
					}
					hint={
						all.length === 0
							? 'Les avoirs apparaissent ici dès que tu en émets un depuis une facture.'
							: 'Vérifie l’orthographe de ta recherche.'
					}
				>
					{text.trim() !== '' && (
						<Button variant="outline" size="sm" onClick={() => change('')}>
							Effacer la recherche
						</Button>
					)}
				</EmptyState>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<caption className="sr-only">Liste des avoirs</caption>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead scope="col" className="pl-4">
									Numéro
								</TableHead>
								<TableHead scope="col">Facture d’origine</TableHead>
								<TableHead scope="col">Client</TableHead>
								<TableHead scope="col" className="max-lg:hidden">
									Date
								</TableHead>
								<TableHead scope="col" className="max-xl:hidden">
									Motif
								</TableHead>
								<TableHead scope="col">Type</TableHead>
								<TableHead scope="col" className="pr-4 text-right">
									Montant crédité
								</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{rows.map((row) => (
								<TableRow
									key={row.note.id}
									className="relative focus-within:bg-muted/50"
								>
									<TableCell className="py-2.5 pl-4">
										<Link
											to={buildOrgPath(
												activeOrganization.slug,
												'/crm/avoirs/$creditNoteId',
											)}
											params={{ creditNoteId: row.note.id }}
											className="font-medium outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
										>
											{row.note.number ?? 'Brouillon'}
										</Link>
									</TableCell>
									<TableCell>
										{row.source ? (
											<InvoiceLink
												invoiceId={row.source.id}
												className="relative z-10"
											>
												{row.sourceLabel}
											</InvoiceLink>
										) : (
											<span className="text-muted-foreground">—</span>
										)}
									</TableCell>
									<TableCell>
										<AccountLink
											customerId={row.note.customer_id}
											className="relative z-10 block max-w-48 truncate"
										>
											{row.customerName || 'Client inconnu'}
										</AccountLink>
									</TableCell>
									<TableCell className="tabular whitespace-nowrap max-lg:hidden">
										{formatDocumentDate(
											row.note.issued_at ?? row.note.created_at,
										)}
									</TableCell>
									<TableCell className="max-w-72 truncate text-muted-foreground max-xl:hidden">
										{row.note.notes ?? '—'}
									</TableCell>
									<TableCell>
										<DocBadge
											tone={
												row.kindLabel === 'Annulation totale'
													? 'neutral'
													: 'warn'
											}
										>
											{row.kindLabel}
										</DocBadge>
									</TableCell>
									<TableCell className="pr-4 text-right font-medium">
										<Money cents={row.totalCents} />
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
