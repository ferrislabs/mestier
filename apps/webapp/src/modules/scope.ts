import { moduleLandingPath } from '#/modules/landing'
import { splitOrgPath } from '#/modules/org-path'
import { resolveModule } from '#/modules/resolve-module'
import type { AppModule, ModuleSection, ModuleTab } from '#/modules/types'

export interface Scope {
	/** Libellé du scope actif : la section quand il y en a une, sinon le module. */
	label: string
	/** Second niveau du scope actif, rendu en onglets horizontaux. */
	tabs: ModuleTab[]
}

/**
 * Section de nav latérale qui couvre un chemin donné : la plus profonde dont le
 * chemin est un préfixe du chemin courant.
 */
export function resolveSection(
	module: AppModule,
	path: string,
): ModuleSection | undefined {
	let best: ModuleSection | undefined

	for (const section of module.sections) {
		if (section.status === 'coming-soon') continue
		if (path !== section.to && !path.startsWith(`${section.to}/`)) continue
		if (!best || section.to.length > best.to.length) best = section
	}

	return best
}

export function resolveScope(pathname: string): Scope {
	const path = splitOrgPath(pathname).path
	const module = resolveModule(path)
	const section = resolveSection(module, path)

	return {
		label: section?.label ?? module.label,
		tabs: section?.tabs ?? [],
	}
}

/**
 * Équivalent d'un chemin dans une autre organisation.
 *
 * Un écran de liste se transpose tel quel ; un écran d'entité non — l'identifiant
 * d'un client n'existe pas dans l'organisation cible. On remonte alors à la
 * section la plus profonde qui reste valide.
 */
export function crossOrganizationPath(path: string): string {
	const module = resolveModule(path)
	const section = resolveSection(module, path)

	if (!section) {
		return module.hasOverview ? module.basePath : moduleLandingPath(module.id)
	}

	return path === section.to ? path : section.to
}
