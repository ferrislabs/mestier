// MOCK: workflow placements and archive — no backend endpoint yet. Replace with `folder_id`, `tags` and `archived_at` on the workflow response.
import { normalizeText } from '#/pages/automation/lib/text'

export interface WorkflowPlacement {
	folderId: string | null
	tags: readonly string[]
}

export const NO_PLACEMENT: WorkflowPlacement = { folderId: null, tags: [] }

const STORAGE_KEY = 'mestier:automation-workflow-placements:v1'

const SEEDED_BY_NAME: Readonly<Record<string, WorkflowPlacement>> = {
	'relance des devis': { folderId: 'ventes-relances', tags: ['Ventes'] },
	'devis accepte → creation du projet': {
		folderId: 'ventes',
		tags: ['Ventes', 'Projets'],
	},
	'ticket urgent → canal #support': { folderId: 'support', tags: ['Support'] },
	'facture en retard → relance client': {
		folderId: 'facturation',
		tags: ['Facturation'],
	},
	'fin d’intervention → rapport au client': {
		folderId: 'chantiers',
		tags: ['Interventions', 'Clients'],
	},
}

const FALLBACK_PLACEMENTS: readonly WorkflowPlacement[] = [
	{ folderId: 'ventes', tags: ['Ventes'] },
	{ folderId: 'ventes-relances', tags: ['Ventes', 'Clients'] },
	{ folderId: 'facturation', tags: ['Facturation'] },
	{ folderId: 'support', tags: ['Support'] },
	{ folderId: 'chantiers', tags: ['Interventions'] },
	{ folderId: 'chantiers-entretien', tags: ['Interventions', 'Projets'] },
	{ folderId: null, tags: [] },
]

interface Persisted {
	placements: Record<string, WorkflowPlacement>
	archived: string[]
}

function readPersisted(): Persisted {
	try {
		const stored = localStorage.getItem(STORAGE_KEY)
		if (stored) return JSON.parse(stored) as Persisted
	} catch {
		return { placements: {}, archived: [] }
	}
	return { placements: {}, archived: [] }
}

const initial = readPersisted()
const overrides = new Map<string, WorkflowPlacement>(
	Object.entries(initial.placements),
)
const archived = new Set<string>(initial.archived)
const names = new Map<string, string>()
const listeners = new Set<() => void>()

function hash(value: string): number {
	let result = 0
	for (const char of value) result = (result * 31 + char.charCodeAt(0)) >>> 0
	return result
}

export function seededPlacement(id: string, name?: string): WorkflowPlacement {
	const byName = name ? SEEDED_BY_NAME[normalizeText(name)] : undefined
	if (byName) return byName
	return (
		FALLBACK_PLACEMENTS[hash(id) % FALLBACK_PLACEMENTS.length] ?? NO_PLACEMENT
	)
}

export function placementOf(id: string): WorkflowPlacement {
	return overrides.get(id) ?? seededPlacement(id, names.get(id))
}

const placementsRecord = (): Readonly<Record<string, WorkflowPlacement>> =>
	new Proxy(
		{},
		{
			get: (_target, key) =>
				typeof key === 'string' ? placementOf(key) : undefined,
		},
	)

let placementsSnapshot = placementsRecord()
let archivedSnapshot: ReadonlySet<string> = new Set(archived)

function persist(): boolean {
	try {
		localStorage.setItem(
			STORAGE_KEY,
			JSON.stringify({
				placements: Object.fromEntries(overrides),
				archived: [...archived],
			} satisfies Persisted),
		)
		return true
	} catch {
		return false
	}
}

function publish() {
	placementsSnapshot = placementsRecord()
	archivedSnapshot = new Set(archived)
	persist()
	for (const listener of listeners) listener()
}

export function subscribePlacements(listener: () => void) {
	listeners.add(listener)
	return () => {
		listeners.delete(listener)
	}
}

export function readPlacements(): Readonly<Record<string, WorkflowPlacement>> {
	return placementsSnapshot
}

export function readArchived(): ReadonlySet<string> {
	return archivedSnapshot
}

export function rememberWorkflowNames(
	entries: readonly { id: string; name: string }[],
) {
	let changed = false
	for (const entry of entries) {
		if (names.get(entry.id) === entry.name) continue
		names.set(entry.id, entry.name)
		changed = true
	}
	if (changed) publish()
}

export function setPlacementMock(id: string, placement: WorkflowPlacement) {
	overrides.set(id, placement)
	publish()
}

export function archiveWorkflowMock(id: string) {
	archived.add(id)
	publish()
}

export function resetPlacementsMock() {
	overrides.clear()
	archived.clear()
	names.clear()
	publish()
}
