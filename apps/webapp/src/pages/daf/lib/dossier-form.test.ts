import { describe, expect, it } from 'vitest'
import { buildDossierBody, dossierToForm } from '#/pages/daf/lib/dossier-form'

describe('dossier form', () => {
	it('round-trips an empty override to null', () => {
		const values = dossierToForm({
			project_id: 'p1',
			title: 'DAF Parc Montgroux',
			issuer: 'Treyve Paysages',
			project_owner: 'Commune',
			updated_at: null,
		})
		expect(values.projectManager).toBe('')
		expect(buildDossierBody(values)).toEqual({
			title: 'DAF Parc Montgroux',
			issuer: 'Treyve Paysages',
			project_owner: 'Commune',
			project_manager: null,
		})
	})
})
