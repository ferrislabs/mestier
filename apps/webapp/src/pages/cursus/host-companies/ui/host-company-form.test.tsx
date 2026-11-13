import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { HostCompanyForm } from '#/pages/cursus/host-companies/ui/host-company-form'

describe('HostCompanyForm', () => {
	it('shows every field at once without steps', () => {
		render(
			<HostCompanyForm
				submitLabel="Enregistrer"
				isPending={false}
				error={null}
				onSubmit={vi.fn()}
			/>,
		)

		for (const label of [
			"Nom de l'entreprise",
			'Numéro de SIRET',
			'Nom du directeur',
			'Téléphone',
			'URL site internet',
			"Nombre d'employés dans le groupe",
			"Domaines d'activité",
		]) {
			expect(screen.getByLabelText(label)).toBeTruthy()
		}
		expect(
			screen.getByRole('button', { name: 'Ajouter une adresse' }),
		).toBeTruthy()
		expect(screen.getByRole('button', { name: 'Enregistrer' })).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Suivant' })).toBeNull()
	})
})
