import {
	Boxes,
	BriefcaseBusiness,
	CalendarDays,
	CalendarRange,
	FileText,
	KanbanSquare,
	LayoutDashboard,
	ListTree,
	MessagesSquare,
	Package,
	Receipt,
	Settings,
	Users,
} from 'lucide-react'
import type { AppModule } from '#/modules/types'

export const MODULES: AppModule[] = [
	{
		id: 'home',
		label: 'Accueil',
		icon: LayoutDashboard,
		basePath: '/',
		status: 'available',
		hasOverview: true,
		sections: [
			{
				id: 'overview',
				label: 'Vue d’ensemble',
				to: '/',
				icon: LayoutDashboard,
				exact: true,
			},
		],
	},
	{
		id: 'crm',
		label: 'CRM',
		icon: Users,
		basePath: '/crm',
		status: 'available',
		hasOverview: false,
		sections: [
			{
				id: 'customers',
				label: 'Clients',
				to: '/crm/customers',
				icon: Users,
				exact: true,
			},
			{
				id: 'pipeline',
				label: 'Pipeline',
				to: '/crm/customers/pipeline',
				icon: KanbanSquare,
			},
			{ id: 'quotes', label: 'Devis', to: '/crm/quotes', icon: FileText },
			{
				id: 'invoices',
				label: 'Factures',
				to: '/crm/invoices',
				icon: Receipt,
				status: 'coming-soon',
			},
			{
				id: 'catalog',
				label: 'Catalogue',
				to: '/crm/catalog',
				icon: Boxes,
			},
		],
	},
	{
		id: 'hr',
		label: 'RH',
		icon: BriefcaseBusiness,
		basePath: '/hr',
		status: 'available',
		hasOverview: false,
		sections: [
			{ id: 'employees', label: 'Employés', to: '/hr/employees', icon: Users },
		],
	},
	{
		id: 'planning',
		label: 'Planning',
		icon: CalendarRange,
		basePath: '/planning',
		status: 'available',
		hasOverview: false,
		sections: [
			{
				id: 'calendar',
				label: 'Calendrier',
				to: '/planning/calendar',
				icon: CalendarDays,
			},
			{
				id: 'team',
				label: 'Vue équipe',
				to: '/planning/team',
				icon: CalendarRange,
			},
			{
				id: 'tasks',
				label: 'Liste des tâches',
				to: '/planning/tasks',
				icon: ListTree,
			},
			{
				id: 'equipment',
				label: 'Matériel',
				to: '/planning/equipment',
				icon: Package,
			},
		],
	},
	{
		id: 'chat',
		label: 'Discussions',
		icon: MessagesSquare,
		basePath: '/chat',
		status: 'coming-soon',
		hasOverview: false,
		sections: [],
	},
	{
		id: 'settings',
		label: 'Paramètres',
		icon: Settings,
		basePath: '/settings',
		status: 'available',
		hasOverview: true,
		railPlacement: 'utility',
		// `/settings` reste une page unique à ancres : ses sections deviendront des
		// entrées de nav — et l'AnchorNav disparaîtra — à l'étape
		// « settings-as-module ».
		sections: [],
	},
]
