import type { PermissionName } from '#/hooks/use-permissions'

/**
 * The six areas #308 groups permission bits under — matches the issue's own
 * wording ("Planning, Coûts, Commercial, Référence, Chat, Administration").
 */
export type PermissionArea =
	| 'planning'
	| 'costs'
	| 'commercial'
	| 'reference'
	| 'chat'
	| 'administration'

export const PERMISSION_AREA_LABELS: Record<PermissionArea, string> = {
	planning: 'Planning',
	costs: 'Coûts',
	commercial: 'Commercial',
	reference: 'Référence',
	chat: 'Discussion',
	administration: 'Administration',
}

export interface PermissionDescriptor {
	name: PermissionName
	area: PermissionArea
	/** A short, active-voice label — what somebody with the bit can do. */
	label: string
	/** One sentence, plain enough for an artisan who has never seen the word "permission". */
	description: string
	/**
	 * Said on the spot when this bit only really means something next to
	 * another one — #308's own example: `VIEW_REPORTS` without `VIEW_COST`
	 * gives hours, not money, and that combination is worth knowing about
	 * rather than discovering by accident.
	 */
	note?: string
}

/**
 * Every named bit (`mestier_core::domain::role::Permissions::NAMED`), with
 * the French sentence #308 asks for. `permission-catalog.test.ts` locks
 * this list to exactly the 15 names `PermissionName` carries, so a bit
 * added on the backend fails loudly here instead of showing up unlabeled.
 *
 * `VIEW_COST` and `MANAGE_COST` are worded to read differently at a
 * glance, on purpose: granting the wrong one is how payroll leaks.
 */
export const PERMISSION_CATALOG: PermissionDescriptor[] = [
	// Planning
	{
		name: 'VIEW_PLANNING',
		area: 'planning',
		label: 'Voir le planning',
		description:
			'Consulter le calendrier, les tâches et les projets planifiés.',
	},
	{
		name: 'MANAGE_PLANNING',
		area: 'planning',
		label: 'Modifier le planning',
		description: 'Créer, déplacer et modifier les tâches, projets et créneaux.',
	},

	// Coûts
	{
		name: 'VIEW_COST',
		area: 'costs',
		label: 'Voir ce que coûtent les personnes et le matériel',
		description:
			'Voir les taux horaires, salaires et coûts planifiés dans les rapports.',
		note: 'Avec « Voir la rentabilité » mais sans ce bit, un rapport montre les heures, jamais les montants.',
	},
	{
		name: 'MANAGE_COST',
		area: 'costs',
		label: 'Modifier ces coûts',
		description:
			'Modifier les taux horaires, salaires et bases de coût des personnes.',
	},
	{
		name: 'VIEW_REPORTS',
		area: 'costs',
		label: 'Voir la rentabilité',
		description:
			'Consulter le rapport de rentabilité : heures planifiées toujours, montants seulement avec « Voir ce que coûtent les personnes et le matériel ».',
	},

	// Commercial
	{
		name: 'MANAGE_CUSTOMERS',
		area: 'commercial',
		label: 'Gérer les clients',
		description: 'Créer, modifier et archiver les fiches clients.',
	},
	{
		name: 'MANAGE_QUOTES',
		area: 'commercial',
		label: 'Gérer les devis',
		description: 'Créer, modifier et envoyer les devis.',
	},

	// Référence
	{
		name: 'MANAGE_REFERENCE',
		area: 'reference',
		label: 'Gérer le référentiel',
		description:
			'Gérer le catalogue (produits, tarifs), le matériel et les absences.',
	},

	// Discussion
	{
		name: 'VIEW_CHANNEL',
		area: 'chat',
		label: 'Voir les canaux',
		description: 'Voir les canaux de discussion et leur historique.',
	},
	{
		name: 'SEND_MESSAGES',
		area: 'chat',
		label: 'Écrire dans les canaux',
		description: 'Envoyer des messages dans les canaux de discussion.',
	},
	{
		name: 'MANAGE_CHANNELS',
		area: 'chat',
		label: 'Gérer les canaux',
		description: 'Créer, renommer et archiver les canaux de discussion.',
	},
	{
		name: 'MANAGE_WEBHOOKS',
		area: 'chat',
		label: 'Gérer les automatisations de discussion',
		description: 'Configurer les webhooks et intégrations liés aux canaux.',
	},

	// Administration
	{
		name: 'MANAGE_ORG',
		area: 'administration',
		label: "Gérer l'organisation",
		description:
			"Modifier les informations de l'organisation (identité légale, paramètres).",
	},
	{
		name: 'MANAGE_MEMBERS',
		area: 'administration',
		label: 'Gérer les membres',
		description: "Ajouter, modifier et retirer des membres de l'équipe.",
	},
	{
		name: 'MANAGE_ROLES',
		area: 'administration',
		label: 'Gérer les rôles',
		description:
			'Créer des rôles, modifier leurs permissions et les attribuer aux membres.',
		note: "Un rôle qui perd ce bit peut encore agir avec les permissions qu'il a déjà, mais plus modifier aucun rôle — de quoi s'enfermer dehors si c'est le seul rôle qui l'a encore.",
	},
]

/** Bit names grouped under their area, in `PERMISSION_AREA_LABELS`' order —
 * what the role editor renders, area by area, instead of a flat list of 15
 * checkboxes. */
export function permissionsByArea(): Array<{
	area: PermissionArea
	label: string
	permissions: PermissionDescriptor[]
}> {
	return (Object.keys(PERMISSION_AREA_LABELS) as PermissionArea[]).map(
		(area) => ({
			area,
			label: PERMISSION_AREA_LABELS[area],
			permissions: PERMISSION_CATALOG.filter((p) => p.area === area),
		}),
	)
}

export function permissionDescriptor(
	name: PermissionName,
): PermissionDescriptor | undefined {
	return PERMISSION_CATALOG.find((p) => p.name === name)
}
