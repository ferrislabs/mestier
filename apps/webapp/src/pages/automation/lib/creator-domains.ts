import type { LucideIcon } from 'lucide-react'
import {
	FALLBACK_HUE,
	FALLBACK_ICON,
	GROUP_APPEARANCE,
} from '#/pages/automation/lib/connector-copy'
import type {
	CreatorCategory,
	CreatorKind,
} from '#/pages/automation/lib/creator'

const DOMAIN_DESCRIPTIONS: Record<string, string> = {
	Achats: 'Fournisseurs et factures fournisseur',
	Clients: 'Clients, contacts et contextes',
	Devis: 'Créer, suivre et planifier des devis',
	Discussions: 'Canaux, fils, messages et permissions',
	DOE: 'Dossiers et modèles d’ouvrages exécutés',
	Entretien: 'Contrats, grilles et interventions d’entretien',
	Factures: 'Émettre, suivre et encaisser des factures',
	Planning: 'Projets, tâches, modèles, récurrences et rythmes de travail',
	Rapports: 'Rentabilité et heures travaillées',
	Référentiel: 'Produits, équipements, tarifs, coûts et absences',
	Support: 'Tickets, brouillons, badges et notes',
	Terrain: 'Pointage, fin de journée et signalements de terrain',
}

export const OTHER_DOMAIN_ID = 'Autres'

export interface DomainCopy {
	description: string
	icon: LucideIcon
	hue: number
}

export function domainCopy(group: string): DomainCopy {
	const appearance = GROUP_APPEARANCE[group]
	return {
		description: DOMAIN_DESCRIPTIONS[group] ?? '',
		icon: appearance?.icon ?? FALLBACK_ICON,
		hue: appearance?.hue ?? FALLBACK_HUE,
	}
}

function byLabel(a: CreatorKind, b: CreatorKind): number {
	return a.label.localeCompare(b.label, 'fr') || a.id.localeCompare(b.id)
}

export function creatorDomains(
	kinds: readonly CreatorKind[],
): CreatorCategory[] {
	const byDomain = new Map<string, CreatorKind[]>()
	for (const kind of kinds) {
		const id = kind.group ?? OTHER_DOMAIN_ID
		const bucket = byDomain.get(id)
		if (bucket) bucket.push(kind)
		else byDomain.set(id, [kind])
	}
	return [...byDomain]
		.map(([id, grouped]) => ({
			id,
			label: id,
			description: domainCopy(id).description,
			icon: domainCopy(id).icon,
			kinds: grouped.sort(byLabel),
		}))
		.sort((a, b) => a.label.localeCompare(b.label, 'fr'))
}
