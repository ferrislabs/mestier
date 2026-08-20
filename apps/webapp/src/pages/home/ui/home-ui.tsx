import { Package, Receipt, TrendingUp, Users } from 'lucide-react'
import type * as React from 'react'
import {
	EntityAvatar,
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'

interface HomeUIProps {
	userName?: string
	stats: {
		customers: number
		inventory: number
		invoices: number
		revenueMonth: number
	}
	/** The self-service pointage card, composed in by the feature layer. */
	todayTasks?: React.ReactNode
}

export function HomeUI({ userName, stats, todayTasks }: HomeUIProps) {
	return (
		<PageShell>
			<PageHeader
				title={userName ? `Bonjour, ${userName}` : 'Tableau de bord'}
				description="Voici un résumé de votre activité. Gérez vos clients, devis et factures en un clin d'œil."
			/>

			<section className="flex flex-col gap-4">
				<p className="text-sm text-muted-foreground">Aperçu général</p>
				<div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
					<MetricCard
						label="Clients"
						value={stats.customers.toString()}
						hint="Total enregistrés"
						icon={<Users className="size-4" />}
					/>
					<MetricCard
						label="Stock"
						value={stats.inventory.toString()}
						hint="Articles en inventaire"
						icon={<Package className="size-4" />}
					/>
					<MetricCard
						label="Factures"
						value={stats.invoices.toString()}
						hint="En attente de paiement"
						icon={<Receipt className="size-4" />}
					/>
					<MetricCard
						label="CA du mois"
						value={`${stats.revenueMonth.toLocaleString('fr-FR')} €`}
						hint="Revenus générés ce mois"
						trend="+12,6%"
						icon={<TrendingUp className="size-4" />}
					/>
				</div>
			</section>

			<section className="grid grid-cols-1 gap-4 lg:grid-cols-3">
				<SectionCard className="lg:col-span-2">
					<SectionHeader
						title="Activité récente"
						description="Derniers événements de votre espace"
					/>
					<ul className="divide-y">
						<ActivityRow
							letter="M"
							tone="success"
							title="Marie Leroy"
							badge="client"
							badgeTone="success"
							subtitle="Nouveau client ajouté · Bordeaux"
							meta="il y a 2 j"
						/>
						<ActivityRow
							letter="P"
							tone="warning"
							title="Projet Dupont"
							badge="mise à jour"
							badgeTone="warning"
							subtitle="Fiche client mise à jour · Lyon"
							meta="il y a 5 j"
						/>
						<ActivityRow
							letter="C"
							tone="brand"
							title="Cloud IAM"
							badge="prospect"
							badgeTone="brand"
							subtitle="Nouveau client ajouté · Toulouse"
							meta="il y a 1 sem"
						/>
					</ul>
				</SectionCard>

				{todayTasks ? <div className="lg:col-span-1">{todayTasks}</div> : null}
			</section>
		</PageShell>
	)
}

interface ActivityRowProps {
	letter: string
	tone: 'brand' | 'success' | 'warning' | 'neutral'
	title: string
	badge: string
	badgeTone: 'brand' | 'success' | 'warning' | 'neutral'
	subtitle: string
	meta: string
}

function ActivityRow({
	letter,
	tone,
	title,
	badge,
	badgeTone,
	subtitle,
	meta,
}: ActivityRowProps) {
	return (
		<li className="flex items-center gap-3 px-5 py-3">
			<EntityAvatar tone={tone}>{letter}</EntityAvatar>
			<div className="min-w-0 flex-1">
				<div className="flex items-center gap-2">
					<p className="truncate font-medium">{title}</p>
					<StatusBadge tone={badgeTone}>{badge}</StatusBadge>
				</div>
				<p className="truncate text-xs text-muted-foreground">{subtitle}</p>
			</div>
			<span className="text-xs text-muted-foreground">{meta}</span>
		</li>
	)
}
