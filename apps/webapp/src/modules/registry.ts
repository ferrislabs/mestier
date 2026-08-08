import {
	Boxes,
	BriefcaseBusiness,
	CalendarRange,
	FileText,
	KanbanSquare,
	LayoutDashboard,
	Link2,
	ListTree,
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
					{ title: 'Devis', to: '/crm/quotes', icon: FileText },
					{
						title: 'Factures',
						to: '/crm/invoices',
						icon: Receipt,
						disabled: true,
					},
				],
			},
			{
				label: 'Configuration du module',
				items: [
					{ title: 'Configuration', to: '/settings', hash: 'crm', icon: Boxes },
				],
			},
		],
	},
	{
		id: 'hr',
		label: 'RH',
		icon: BriefcaseBusiness,
		basePath: '/hr',
		enabled: true,
		nav: [
			{
				label: 'Équipe',
				items: [{ title: 'Employés', to: '/hr/employees', icon: Users }],
			},
		],
	},
	{
		id: 'planning',
		label: 'Planning',
		icon: CalendarRange,
		basePath: '/planning',
		enabled: true,
		nav: [
			{
				label: 'Équipe',
				items: [
					{ title: 'Vue équipe', to: '/planning/team', icon: CalendarRange },
					{ title: 'Liste des tâches', to: '/planning/tasks', icon: ListTree },
				],
			},
		],
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
