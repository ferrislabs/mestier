import { Link } from '@tanstack/react-router'
import {
	Activity,
	ArrowLeft,
	Building2,
	FilePlus2,
	type LucideIcon,
	MapPin,
	Receipt,
	Users,
} from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueNav } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import {
	CLIENT_TABS,
	type ClientTab,
	isClientTab,
} from '#/pages/customers/lib/rubriques'
import type { AccountStatus } from '#/pages/customers/types'
import { NameAvatar } from '#/pages/customers/ui/avatars'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { AccountStatusBadge } from '#/pages/customers/ui/status-badges'

const ICONS: Record<ClientTab, LucideIcon> = {
	identite: Building2,
	contacts: Users,
	adresses: MapPin,
	activite: Activity,
}

export function BackLink({ organizationSlug }: { organizationSlug: string }) {
	return (
		<Link
			to={buildOrgPath(organizationSlug, '/crm/clients')}
			className="-ml-1 inline-flex items-center gap-1.5 rounded-sm px-1 text-[13px] text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
		>
			<ArrowLeft className="size-4" aria-hidden />
			Clients
		</Link>
	)
}

export function CustomerSidebar({
	organizationSlug,
	customerId,
	name,
	hue,
	status,
	kindLabel,
	tab,
	summaries,
}: {
	organizationSlug: string
	customerId: string
	name: string
	hue: number
	status: AccountStatus
	kindLabel: string
	tab: ClientTab
	summaries: Record<ClientTab, string>
}) {
	const archived = status === 'archived'
	return (
		<aside
			aria-label="Fiche du client"
			className="flex flex-col border-border border-b bg-card lg:min-h-0 lg:border-r lg:border-b-0"
		>
			<div className="shrink-0 border-border border-b px-4 py-4">
				<BackLink organizationSlug={organizationSlug} />
				<div className="mt-3 flex items-center gap-3">
					<NameAvatar name={name} hue={hue} size="xl" />
					<div className="min-w-0">
						<h1 className="break-words font-semibold text-lg leading-tight tracking-tight">
							{name}
						</h1>
						<div className="mt-1.5 flex flex-wrap items-center gap-1.5">
							<AccountStatusBadge status={status} />
							<StatusBadge tone="neutral">{kindLabel}</StatusBadge>
						</div>
					</div>
				</div>
				{!archived && (
					<div className="mt-3 flex flex-wrap items-center gap-2">
						<RequirePermission permission="MANAGE_QUOTES">
							<Button variant="outline" size="sm" asChild>
								<Link
									to={buildOrgPath(organizationSlug, '/crm/devis/nouveau')}
									search={{ client: customerId }}
								>
									<FilePlus2 aria-hidden /> Nouveau devis
								</Link>
							</Button>
						</RequirePermission>
						<RequirePermission permission="MANAGE_INVOICES">
							<Button variant="outline" size="sm" asChild>
								<Link
									to={buildOrgPath(organizationSlug, '/crm/factures/nouvelle')}
									search={{ client: customerId }}
								>
									<Receipt aria-hidden /> Nouvelle facture
								</Link>
							</Button>
						</RequirePermission>
					</div>
				)}
			</div>
			<RubriqueNav
				label="Rubriques du client"
				selected={tab}
				items={CLIENT_TABS.map((item) => ({
					id: item.id,
					label: item.label,
					icon: ICONS[item.id],
					summary: summaries[item.id],
				}))}
				renderLink={(item, props) => (
					<Link
						to={buildOrgPath(organizationSlug, '/crm/clients/$customerId')}
						params={{ customerId }}
						search={{ onglet: isClientTab(item.id) ? item.id : 'identite' }}
						{...props}
					/>
				)}
			/>
		</aside>
	)
}
