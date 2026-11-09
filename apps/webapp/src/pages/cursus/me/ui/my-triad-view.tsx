import { ArrowLeft } from 'lucide-react'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { MyTriadDetail } from '#/pages/cursus/me/feature/use-my-cursus'
import { ROLE_LABELS, readOnlySince } from '#/pages/cursus/me/lib/labels'
import {
	CONTRACT_KIND_LABELS,
	formatDate,
} from '#/pages/cursus/shared/lib/format'

interface MyTriadViewProps {
	triad: MyTriadDetail
	onBack: () => void
}

export function MyTriadView({ triad, onBack }: MyTriadViewProps) {
	const since = readOnlySince(triad.access)

	return (
		<PageShell>
			<PageHeader
				leading={
					<Button
						type="button"
						variant="ghost"
						size="icon"
						aria-label="Retour aux trinômes"
						onClick={onBack}
					>
						<ArrowLeft />
					</Button>
				}
				title={triad.apprentice_name}
				description={triad.apprentice_email}
				actions={
					<>
						<Badge variant="outline">{ROLE_LABELS[triad.role]}</Badge>
						{since ? (
							<Badge variant="secondary">Historique, lecture seule</Badge>
						) : null}
					</>
				}
			/>
			<SectionCard>
				<dl className="grid gap-4 p-5 text-[13px] sm:grid-cols-3">
					<div>
						<dt className="text-muted-foreground">Promotion</dt>
						<dd>
							{triad.promotion_label}
							<span className="block text-muted-foreground">
								{triad.training_name} · {triad.promotion_start_year}-
								{triad.promotion_end_year}
							</span>
						</dd>
					</div>
					<div>
						<dt className="text-muted-foreground">Tuteur pédagogique</dt>
						<dd>{triad.pedagogic_tutor_name ?? '—'}</dd>
					</div>
					<div>
						<dt className="text-muted-foreground">Statut</dt>
						<dd>
							{triad.closed_at ? (
								<>
									Clôturé le {formatDate(triad.closed_at)}
									{triad.closure_reason_label
										? ` · ${triad.closure_reason_label}`
										: ''}
									{triad.closure_note ? (
										<span className="block text-muted-foreground">
											{triad.closure_note}
										</span>
									) : null}
								</>
							) : (
								'En cours'
							)}
							{since ? (
								<span className="block text-muted-foreground">
									Accès en lecture seule depuis le {formatDate(since)}
								</span>
							) : null}
						</dd>
					</div>
				</dl>
			</SectionCard>
			<section className="grid gap-3">
				<h2 className="font-semibold">Contrats</h2>
				{triad.contracts.length === 0 ? (
					<p className="text-[13px] text-muted-foreground">
						Aucun contrat visible.
					</p>
				) : (
					triad.contracts.map((contract) => (
						<article
							key={contract.id}
							className="grid gap-3 rounded-lg border bg-card p-4"
						>
							<div className="flex flex-wrap items-start justify-between gap-3">
								<div>
									<h3 className="font-medium">{contract.host_company_name}</h3>
									<p className="text-[13px] text-muted-foreground">
										{formatDate(contract.start_date)} →{' '}
										{formatDate(contract.end_date)}
										{contract.number ? ` · n° ${contract.number}` : ''}
									</p>
								</div>
								<Badge variant="outline">
									{CONTRACT_KIND_LABELS[contract.kind]}
								</Badge>
							</div>
							<dl className="grid gap-1 text-[13px] sm:grid-cols-2">
								{contract.tutors.map((tutor) => (
									<div key={`${tutor.rank}-${tutor.name}`}>
										<dt className="text-muted-foreground">
											{tutor.rank === 'main'
												? 'Tuteur principal'
												: 'Tuteur secondaire'}
										</dt>
										<dd>{tutor.name}</dd>
									</div>
								))}
							</dl>
							{contract.closed_at ? (
								<div className="rounded-md bg-muted px-3 py-2 text-[13px]">
									<p>
										Clôturé le {formatDate(contract.closed_at)}
										{contract.closure_reason_label
											? ` · ${contract.closure_reason_label}`
											: ''}
									</p>
									{contract.closure_note ? (
										<p className="text-muted-foreground">
											{contract.closure_note}
										</p>
									) : null}
								</div>
							) : null}
						</article>
					))
				)}
			</section>
		</PageShell>
	)
}
