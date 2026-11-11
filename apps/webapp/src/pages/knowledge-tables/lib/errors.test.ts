import { describe, expect, it } from 'vitest'
import { tableErrorMessage } from '#/pages/knowledge-tables/lib/errors'

function failure(status: number, message: string) {
	return Object.assign(new Error(message), { status })
}

describe('tableErrorMessage', () => {
	it('is null without error', () => {
		expect(tableErrorMessage(null)).toBeNull()
	})

	it('says the known refusals in French', () => {
		expect(
			tableErrorMessage(
				failure(
					409,
					'name: a column with this name already exists in the table',
				),
			),
		).toBe('Une colonne porte déjà ce nom dans la table.')
		expect(
			tableErrorMessage(failure(409, 'views: a table keeps at least one view')),
		).toBe('Une table garde au moins une vue.')
		expect(
			tableErrorMessage(
				failure(
					409,
					'kind: a stored value does not fit the new type; nothing was changed',
				),
			),
		).toBe('Une valeur ne tient pas dans le nouveau type : rien n’a changé.')
		expect(
			tableErrorMessage(
				failure(422, 'values.0193: the text is too long for the column'),
			),
		).toBe('Le texte est trop long pour la colonne.')
	})

	it('says a permission refusal plainly', () => {
		expect(tableErrorMessage(failure(403, 'Forbidden'))).toBe(
			"Vous n'avez plus la permission nécessaire pour cette action.",
		)
	})

	it('keeps an unknown message as is', () => {
		expect(tableErrorMessage(failure(422, 'something odd'))).toBe(
			'something odd',
		)
	})
})
