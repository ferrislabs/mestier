import { describe, expect, it } from 'vitest'
import {
	buildCredentialFieldErrors,
	credentialFormDefaults,
	credentialFormFromRow,
	credentialPayload,
	emptyCredentialForm,
	fieldType,
	isMultiline,
	visibleCredentialFields,
} from '#/pages/automation/lib/credential-form'
import { credentialRow } from '#/pages/automation/lib/credential-rows'
import {
	BEARER_TOKEN,
	CUSTOM_AUTH,
	credential,
	field,
	HTTP_HEADER,
	KINDS,
	OAUTH2_AUTHORIZATION_CODE,
	SMTP,
	SSH,
} from '#/pages/automation/test/credential-fixtures'

function values(data: Record<string, string>, name = 'Une identification') {
	return { ...emptyCredentialForm('k'), name, data }
}

describe('fieldType', () => {
	it('reads the kind of control the server asks for', () => {
		expect(fieldType(field('a', 'A', { kind: 'password' }))).toBe('password')
		expect(fieldType(field('a', 'A', { kind: 'boolean' }))).toBe('boolean')
	})

	it('treats a kind of control it does not know as a text', () => {
		expect(fieldType(field('a', 'A', { kind: 'colour' }))).toBe('text')
	})

	it('tells the multi-line controls', () => {
		expect(isMultiline(field('a', 'A', { kind: 'textarea' }))).toBe(true)
		expect(isMultiline(field('a', 'A', { kind: 'json' }))).toBe(true)
		expect(isMultiline(field('a', 'A', { kind: 'text' }))).toBe(false)
	})
})

describe('visibleCredentialFields', () => {
	const names = (data: Record<string, string>) =>
		visibleCredentialFields(SSH.fields, data).map((f) => f.name)

	it('uses the default of the controlling field while nothing is chosen', () => {
		expect(names({})).toEqual([
			'host',
			'port',
			'username',
			'method',
			'private_key',
			'passphrase',
		])
	})

	it('follows the choice made on the controlling field', () => {
		expect(names({ method: 'password' })).toEqual([
			'host',
			'port',
			'username',
			'method',
			'password',
		])
	})
})

describe('credentialFormDefaults', () => {
	it('prefills the defaults of the server, never for a secret', () => {
		expect(credentialFormDefaults(SMTP)).toEqual({
			port: '465',
			security: 'ssl',
		})
		expect(credentialFormDefaults(HTTP_HEADER)).toEqual({
			header_name: 'X-API-Key',
		})
	})

	it('has none without a kind', () => {
		expect(credentialFormDefaults(undefined)).toEqual({})
	})
})

describe('credentialFormFromRow', () => {
	it('prefills the readable values the server returns, over the defaults', () => {
		const row = credentialRow(
			credential({
				kind: 'smtp',
				name: 'Relais',
				description: 'Mails sortants',
				data: { host: 'smtp.exemple.fr', port: 587, security: 'starttls' },
				secrets_set: ['password'],
			}),
			KINDS,
		)

		expect(credentialFormFromRow(row, SMTP)).toEqual({
			kind: 'smtp',
			name: 'Relais',
			description: 'Mails sortants',
			origin: 'supplied',
			data: { host: 'smtp.exemple.fr', port: '587', security: 'starttls' },
		})
	})

	it('writes a boolean as the word the form works with', () => {
		const kind = { fields: [field('ssl', 'SSL', { kind: 'boolean' })] }
		const row = credentialRow(credential({ data: { ssl: true } }), KINDS)

		expect(credentialFormFromRow(row, kind).data).toEqual({ ssl: 'true' })
	})

	it('never reads a secret back', () => {
		const row = credentialRow(
			credential({ data: { token: 'oops' }, secrets_set: ['token'] }),
			KINDS,
		)

		expect(credentialFormFromRow(row, BEARER_TOKEN).data).toEqual({})
	})
})

describe('credentialPayload — creation', () => {
	it('sends the prefilled defaults along with what was typed, trimmed', () => {
		expect(
			credentialPayload(HTTP_HEADER, {
				header_name: ' X-API-Key ',
				header_value: 'sk-123',
			}),
		).toEqual({ header_name: 'X-API-Key', header_value: 'sk-123' })
	})

	it('sends a secret exactly as typed', () => {
		expect(credentialPayload(BEARER_TOKEN, { token: ' a b ' })).toEqual({
			token: ' a b ',
		})
	})

	it('leaves out an empty optional value', () => {
		expect(
			credentialPayload(OAUTH2_AUTHORIZATION_CODE, {
				authorization_url: 'https://a.fr/auth',
				token_url: 'https://a.fr/token',
				client_id: 'id',
				client_secret: 's',
				scopes: '',
				pkce: 'yes',
			}),
		).toEqual({
			authorization_url: 'https://a.fr/auth',
			token_url: 'https://a.fr/token',
			client_id: 'id',
			client_secret: 's',
			pkce: 'yes',
		})
	})

	it('sends a number as a number and a boolean as a boolean', () => {
		const kind = {
			fields: [
				field('port', 'Port', { kind: 'number' }),
				field('ssl', 'SSL', { kind: 'boolean' }),
			],
		}

		expect(credentialPayload(kind, { port: '465', ssl: 'false' })).toEqual({
			port: 465,
			ssl: false,
		})
	})

	it('sends only the fields the current choices show', () => {
		expect(
			credentialPayload(SSH, {
				host: 'srv',
				port: '22',
				username: 'deploy',
				method: 'password',
				password: 'pw',
				private_key: 'left over',
			}),
		).toEqual({
			host: 'srv',
			port: 22,
			username: 'deploy',
			method: 'password',
			password: 'pw',
		})
	})
})

describe('credentialPayload — edition', () => {
	const initial = {
		host: 'smtp.exemple.fr',
		port: '465',
		security: 'ssl',
		username: 'alice',
	}

	it('sends nothing when nothing changed', () => {
		expect(credentialPayload(SMTP, initial, initial)).toEqual({})
	})

	it('sends only the readable fields that changed', () => {
		expect(
			credentialPayload(SMTP, { ...initial, port: '587' }, initial),
		).toEqual({ port: 587 })
	})

	it('sends only the secret that was replaced, not the others', () => {
		const header = { header_name: 'X-API-Key' }

		expect(
			credentialPayload(
				HTTP_HEADER,
				{ ...header, header_value: 'new' },
				header,
			),
		).toEqual({ header_value: 'new' })
		expect(credentialPayload(HTTP_HEADER, { ...header }, header)).toEqual({})
	})

	it('sends an emptied optional field as an empty value so the server clears it', () => {
		expect(
			credentialPayload(SMTP, { ...initial, username: '' }, initial),
		).toEqual({ username: '' })
	})
})

describe('buildCredentialFieldErrors', () => {
	it('asks for a name', () => {
		expect(buildCredentialFieldErrors(values({}, '  '), undefined)).toEqual({
			name: 'Donnez un nom à cette identification.',
		})
	})

	it('asks for every required field the form shows', () => {
		expect(
			buildCredentialFieldErrors(values({ header_name: '' }), HTTP_HEADER),
		).toEqual({
			header_name: 'Champ obligatoire.',
			header_value: 'Champ obligatoire.',
		})
	})

	it('does not ask for a field the current choices hide', () => {
		const errors = buildCredentialFieldErrors(
			values({ host: 'srv', port: '22', username: 'u', method: 'password' }),
			SSH,
		)

		expect(errors).toEqual({ password: 'Champ obligatoire.' })
	})

	it('accepts a stored secret left alone', () => {
		expect(
			buildCredentialFieldErrors(
				values({ header_name: 'X-API-Key' }),
				HTTP_HEADER,
				['header_value'],
			),
		).toEqual({})
	})

	it('asks again for a secret that was never stored', () => {
		expect(
			buildCredentialFieldErrors(
				values({ method: 'password', host: 'h', port: '22', username: 'u' }),
				SSH,
				['private_key'],
			),
		).toEqual({ password: 'Champ obligatoire.' })
	})

	it('wants a whole number for a number', () => {
		expect(
			buildCredentialFieldErrors(
				values({ host: 'smtp', port: '46x5', security: 'ssl' }),
				SMTP,
			),
		).toEqual({ port: 'Nombre entier attendu.' })
	})

	it('wants a JSON object for a JSON field', () => {
		const errorFor = (text: string) =>
			buildCredentialFieldErrors(values({ json: text }), CUSTOM_AUTH).json

		expect(errorFor('{ "headers": { "X-Token": "a" } }')).toBeUndefined()
		expect(errorFor('[1]')).toBe('JSON invalide : un objet est attendu.')
		expect(errorFor('{oops')).toBe('JSON invalide : un objet est attendu.')
	})

	it('never reproaches a boolean', () => {
		const kind = {
			fields: [field('ssl', 'SSL', { kind: 'boolean', required: true })],
		}

		expect(buildCredentialFieldErrors(values({}), kind)).toEqual({})
	})

	it('checks nothing but the name for a secret generated by Mestier', () => {
		expect(
			buildCredentialFieldErrors(
				{ ...values({}), origin: 'generated' },
				BEARER_TOKEN,
			),
		).toEqual({})
	})
})
