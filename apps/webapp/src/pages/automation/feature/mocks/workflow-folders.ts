// MOCK: workflow folders — no backend endpoint yet. Replace with GET /automation/folders (folder tree) once projects exist server-side.
import type { WorkflowFolder } from '#/pages/automation/lib/folders'

export const MOCK_WORKFLOW_FOLDERS: readonly WorkflowFolder[] = [
	{ id: 'ventes', name: 'Ventes', parentId: null },
	{ id: 'ventes-relances', name: 'Relances', parentId: 'ventes' },
	{ id: 'facturation', name: 'Facturation', parentId: null },
	{ id: 'support', name: 'Support client', parentId: null },
	{ id: 'chantiers', name: 'Chantiers', parentId: null },
	{ id: 'chantiers-entretien', name: 'Entretien', parentId: 'chantiers' },
]
