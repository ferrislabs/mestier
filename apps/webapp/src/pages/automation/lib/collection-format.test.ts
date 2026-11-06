import { describe, expect, it } from 'vitest'
import {
	collectionErrorMessage,
	collectionLoadErrorMessage,
	describeSecurity,
	describeSource,
	formatImportDate,
	isCollectionMissing,
	operationCount,
} from '#/pages/automation/lib/collection-format'

function apiError(status: number, message: string) {
	return Object.assign(new Error(message), { status })
}

describe('describeSource', () => {
	it('names the url or the file the collection came from', () => {
		expect(
			describeSource({ kind: 'url', url: 'https://api.exemple.fr/o.json' }),
		).toBe('Importée depuis https://api.exemple.fr/o.json')
		expect(describeSource({ kind: 'file', fileName: 'spec.yaml' })).toBe(
			'Importée depuis le fichier spec.yaml',
		)
	})
})

describe('formatImportDate', () => {
	it('writes the import date in French, and nothing for an unreadable one', () => {
		expect(formatImportDate('2026-11-06T09:30:00Z')).toBe('6 novembre 2026')
		expect(formatImportDate('hier')).toBe('')
	})
})

describe('operationCount', () => {
	it('agrees the noun with the count', () => {
		expect(operationCount(0)).toBe('0 opération')
		expect(operationCount(1)).toBe('1 opération')
		expect(operationCount(12)).toBe('12 opérations')
	})
})

describe('describeSecurity', () => {
	it('tells each kind of scheme the way the OpenAPI spec names it', () => {
		expect(
			describeSecurity({
				name: 'jeton',
				kind: { type: 'http', scheme: 'bearer' },
			}),
		).toBe('jeton (http bearer)')
		expect(
			describeSecurity({
				name: 'cle',
				kind: { type: 'api_key', location: 'header', parameterName: 'X-Key' },
			}),
		).toBe('cle (apiKey, en-tête X-Key)')
		expect(
			describeSecurity({
				name: 'requete',
				kind: { type: 'api_key', location: 'query', parameterName: 'key' },
			}),
		).toBe('requete (apiKey, paramètre key)')
		expect(
			describeSecurity({
				name: 'biscuit',
				kind: { type: 'api_key', location: 'cookie', parameterName: 'sid' },
			}),
		).toBe('biscuit (apiKey, cookie sid)')
		expect(describeSecurity({ name: 'o', kind: { type: 'oauth2' } })).toBe(
			'o (oauth2)',
		)
		expect(
			describeSecurity({ name: 'i', kind: { type: 'open_id_connect' } }),
		).toBe('i (openIdConnect)')
		expect(
			describeSecurity({
				name: 't',
				kind: { type: 'other', declaredType: 'mutualTLS' },
			}),
		).toBe('t (mutualTLS)')
	})
})

describe('collectionErrorMessage', () => {
	it('says nothing when there is no error', () => {
		expect(collectionErrorMessage(null)).toBeNull()
		expect(collectionErrorMessage(undefined)).toBeNull()
	})

	it('explains a collection that is too large', () => {
		expect(
			collectionErrorMessage(
				apiError(
					413,
					'the collection model holds 3000000 bytes, over the limit of 2097152 bytes',
				),
			),
		).toBe(
			'Cette collection est trop volumineuse : le serveur refuse une spec de plus de 2 Mo. Retirez les opérations ou les schémas dont vous n’avez pas besoin, puis réessayez.',
		)
	})

	it('explains that the organization reached its 200 collections', () => {
		expect(
			collectionErrorMessage(
				apiError(
					409,
					'Conflict: an organization holds at most 200 collections',
				),
			),
		).toBe(
			'Cette organisation a atteint la limite de 200 collections. Supprimez-en une avant d’en importer une autre.',
		)
	})

	it('relays the reason of a refused model, without the technical prefixes', () => {
		expect(
			collectionErrorMessage(
				apiError(
					400,
					'Validation failed: invalid collection: operation `getAddress` is declared more than once',
				),
			),
		).toBe(
			'Le serveur refuse cette collection : operation `getAddress` is declared more than once.',
		)
	})

	it('falls back to a plain refusal when the server gave no readable reason', () => {
		expect(collectionErrorMessage(apiError(400, 'HTTP 400: Bad Request'))).toBe(
			'Le serveur refuse cette collection : son modèle n’est pas valide.',
		)
		expect(collectionErrorMessage(apiError(400, ''))).toBe(
			'Le serveur refuse cette collection : son modèle n’est pas valide.',
		)
	})

	it('explains a refused permission and a collection that is gone', () => {
		expect(collectionErrorMessage(apiError(403, 'Forbidden'))).toBe(
			'Vous n’avez pas la permission de gérer les collections.',
		)
		expect(collectionErrorMessage(apiError(404, 'Resource not found'))).toBe(
			'Cette collection n’existe plus.',
		)
	})

	it('keeps a generic message for anything else, a lost connection included', () => {
		const generic = 'Une erreur est survenue. Réessayez dans un instant.'
		expect(collectionErrorMessage(apiError(500, 'Internal'))).toBe(generic)
		expect(collectionErrorMessage(new TypeError('Failed to fetch'))).toBe(
			generic,
		)
		expect(collectionErrorMessage('boom')).toBe(generic)
	})
})

describe('collectionLoadErrorMessage', () => {
	it('says nothing when there is no error', () => {
		expect(collectionLoadErrorMessage(null)).toBeNull()
	})

	it('tells a missing read permission, and keeps a generic message otherwise', () => {
		expect(collectionLoadErrorMessage(apiError(403, 'Forbidden'))).toBe(
			'Vous n’avez pas la permission de voir les collections.',
		)
		expect(collectionLoadErrorMessage(apiError(500, 'Internal'))).toBe(
			'Une erreur est survenue. Réessayez dans un instant.',
		)
	})
})

describe('isCollectionMissing', () => {
	it('is true only for a 404', () => {
		expect(isCollectionMissing(apiError(404, 'Resource not found'))).toBe(true)
		expect(isCollectionMissing(apiError(500, 'Internal'))).toBe(false)
		expect(isCollectionMissing(null)).toBe(false)
		expect(isCollectionMissing(new TypeError('Failed to fetch'))).toBe(false)
	})
})
