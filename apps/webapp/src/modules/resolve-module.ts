import { MODULES } from '#/modules/registry'
import type { AppModule } from '#/modules/types'

export function resolveModule(pathname: string): AppModule {
	const home = MODULES.find((module) => module.id === 'home')
	if (!home) throw new Error('registry: module home manquant')

	let best = home
	for (const module of MODULES) {
		if (module.basePath === '/') continue
		if (!matchesBasePath(pathname, module.basePath)) continue
		if (module.basePath.length > best.basePath.length) best = module
	}

	return best
}

function matchesBasePath(pathname: string, basePath: string): boolean {
	return pathname === basePath || pathname.startsWith(`${basePath}/`)
}
