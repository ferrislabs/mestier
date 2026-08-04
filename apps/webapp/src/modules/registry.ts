import {
	BarChart3,
	Boxes,
	BriefcaseBusiness,
	Eye,
	FileText,
	KanbanSquare,
	LayoutDashboard,
	Link2,
	MessagesSquare,
	Receipt,
	Settings,
	ShieldCheck,
	Users,
} from 'lucide-react'
import type { AppModule, ModuleNavGroup } from '#/modules/types'

export const GLOBAL_NAV_GROUPS: ModuleNavGroup[] = [
	{
		label: 'Configuration',
		items: [
			{ title: 'Paramètres', to: '/settings', icon: Settings },
			{
				title: 'Intégrations',
				to: '/integrations',
				icon: Link2,
				disabled: true,
			},
			{ title: 'Rapports', to: '/reports', icon: BarChart3, disabled: true },
		],
	},
	{
		label: 'Sécurité',
		items: [
			{ title: 'Audit', to: '/audit', icon: Eye, disabled: true },
			{
				title: 'Permissions',
				to: '/permissions',
				icon: ShieldCheck,
				disabled: true,
			},
		],
	},
]

export const MODULES: AppModule[] = [
	{
		id: 'home',
		label: 'Accueil',
		icon: LayoutDashboard,
		basePath: '/',
		enabled: true,
		nav: [
			{
				label: 'Activité',
				items: [
					{ title: 'Accueil', to: '/', icon: LayoutDashboard, exact: true },
				],
			},
		],
	},
	{
		id: 'crm',
		label: 'CRM',
		icon: Users,
		basePath: '/crm',
		enabled: true,
		nav: [
			{
				label: 'Activité',
				items: [
					{ title: 'Clients', to: '/crm/customers', icon: Users, exact: true },
					{
						title: 'Pipeline',
						to: '/crm/customers/pipeline',
						icon: KanbanSquare,
					},
					{ title: 'Catalogue', to: '/catalog', icon: Boxes },
					{ title: 'Devis', to: '/crm/quotes', icon: FileText },
					{
						title: 'Factures',
						to: '/crm/invoices',
						icon: Receipt,
						disabled: true,
					},
				],
			},
		],
	},
	{
		id: 'hr',
		label: 'RH',
		icon: BriefcaseBusiness,
		basePath: '/hr',
		enabled: false,
		nav: [],
	},
	{
		id: 'discussions',
		label: 'Discussions',
		icon: MessagesSquare,
		basePath: '/discussions',
		enabled: false,
		nav: [],
	},
]
