import { Link } from '@tanstack/react-router'
import {
	Boxes,
	FileMinus2,
	FileText,
	HandCoins,
	Landmark,
	type LucideIcon,
	Receipt,
	Settings2,
	SquareKanban,
	Truck,
	Users,
	Wallet,
	Webhook,
} from 'lucide-react'
import type { ReactNode } from 'react'
import type { PermissionName } from '#/hooks/use-permissions'
import { buildOrgPath } from '#/modules/org-path'
import { ENTRY } from '#/pages/crm-nav/lib/crm-path'
import {
	SidebarEntryContent,
	SidebarSection,
	sidebarItemClassName,
} from '#/pages/projet/ui/sidebar-parts'

export interface CrmSidebarCounts {
	clients?: number
	opportunities?: number
	quotesAwaiting?: number
	invoicesOverdue?: number
	creditNotes?: number
	expensesToPay?: number
	stripeToReview?: number
	catalog?: number
	suppliers?: number
}

export interface CrmSidebarProps {
	organizationSlug: string
	active: ReadonlySet<string>
	counts: CrmSidebarCounts
	visible: (permission: PermissionName | null) => boolean
	isCollapsed: (groupId: string) => boolean
	onToggleGroup: (groupId: string) => void
}

interface EntryDef {
	key: string
	to: string
	search?: Record<string, string>
	icon: LucideIcon
	label: string
	count?: keyof CrmSidebarCounts
	permission: PermissionName | null
}

interface GroupDef {
	id: string
	title: string | null
	entries: readonly EntryDef[]
}

const GROUPS: readonly GroupDef[] = [
	{
		id: 'main',
		title: null,
		entries: [
			{
				key: ENTRY.clients,
				to: '/crm/clients',
				icon: Users,
				label: 'Clients',
				count: 'clients',
				permission: 'VIEW_CUSTOMERS',
			},
			{
				key: ENTRY.pipeline,
				to: '/crm/pipeline',
				icon: SquareKanban,
				label: 'Pipeline',
				count: 'opportunities',
				permission: 'VIEW_CUSTOMERS',
			},
		],
	},
	{
		id: 'sales',
		title: 'Ventes',
		entries: [
			{
				key: ENTRY.quotes,
				to: '/crm/devis',
				icon: FileText,
				label: 'Devis',
				count: 'quotesAwaiting',
				permission: 'VIEW_QUOTES',
			},
			{
				key: ENTRY.invoices,
				to: '/crm/factures',
				icon: Receipt,
				label: 'Factures',
				count: 'invoicesOverdue',
				permission: 'VIEW_INVOICES',
			},
			{
				key: ENTRY.creditNotes,
				to: '/crm/avoirs',
				icon: FileMinus2,
				label: 'Avoirs',
				count: 'creditNotes',
				permission: 'VIEW_INVOICES',
			},
		],
	},
	{
		id: 'cash',
		title: 'Trésorerie',
		entries: [
			{
				key: ENTRY.ledger,
				to: '/crm/tresorerie',
				icon: Landmark,
				label: 'Journal',
				permission: 'VIEW_INVOICES',
			},
			{
				key: ENTRY.receipts,
				to: '/crm/tresorerie',
				search: { type: 'recettes' },
				icon: HandCoins,
				label: 'Recettes',
				permission: 'VIEW_INVOICES',
			},
			{
				key: ENTRY.expenses,
				to: '/crm/tresorerie',
				search: { type: 'depenses' },
				icon: Wallet,
				label: 'Dépenses',
				count: 'expensesToPay',
				permission: 'VIEW_INVOICES',
			},
			{
				key: ENTRY.stripe,
				to: '/crm/stripe',
				icon: Webhook,
				label: 'Stripe',
				count: 'stripeToReview',
				permission: 'VIEW_INVOICES',
			},
		],
	},
	{
		id: 'reference',
		title: 'Référentiel',
		entries: [
			{
				key: ENTRY.catalog,
				to: '/crm/catalogue',
				icon: Boxes,
				label: 'Catalogue',
				count: 'catalog',
				permission: 'MANAGE_REFERENCE',
			},
			{
				key: ENTRY.suppliers,
				to: '/crm/fournisseurs',
				icon: Truck,
				label: 'Fournisseurs',
				count: 'suppliers',
				permission: null,
			},
			{
				key: ENTRY.settings,
				to: '/crm/reglages',
				icon: Settings2,
				label: 'Réglages',
				permission: 'MANAGE_ORG',
			},
		],
	},
]

function Entry({
	organizationSlug,
	entry,
	active,
	count,
}: {
	organizationSlug: string
	entry: EntryDef
	active: boolean
	count?: number
}) {
	const Icon = entry.icon
	return (
		<li>
			<Link
				to={buildOrgPath(organizationSlug, entry.to)}
				search={entry.search}
				activeOptions={{ exact: true }}
				aria-current={active ? 'page' : undefined}
				className={sidebarItemClassName(active)}
			>
				<SidebarEntryContent
					leading={<Icon className="size-4" strokeWidth={1.8} aria-hidden />}
					label={entry.label}
					count={count}
				/>
			</Link>
		</li>
	)
}

export function CrmSidebar({
	organizationSlug,
	active,
	counts,
	visible,
	isCollapsed,
	onToggleGroup,
}: CrmSidebarProps) {
	return (
		<nav aria-label="Navigation CRM" className="flex flex-col gap-4">
			{GROUPS.map((group) => {
				const entries = group.entries.filter((entry) =>
					visible(entry.permission),
				)
				if (entries.length === 0) return null
				const items: ReactNode = entries.map((entry) => (
					<Entry
						key={entry.key}
						organizationSlug={organizationSlug}
						entry={entry}
						active={active.has(entry.key)}
						count={entry.count ? counts[entry.count] : undefined}
					/>
				))
				if (group.title === null) {
					return (
						<ul key={group.id} className="flex flex-col gap-px">
							{items}
						</ul>
					)
				}
				return (
					<SidebarSection
						key={group.id}
						title={group.title}
						collapsed={isCollapsed(group.id)}
						onToggle={() => onToggleGroup(group.id)}
					>
						{items}
					</SidebarSection>
				)
			})}
		</nav>
	)
}
