import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { CredentialKind } from '#/hooks/use-automation'
import {
	ConnectorParametersFeature,
	type ConnectorParametersFeatureProps,
} from '#/pages/automation/feature/connector-parameters-feature'
import {
	BEARER_TOKEN,
	credential,
	HTTP_BASIC,
	HTTP_HEADER,
	ODOO_API,
	SMTP,
} from '#/pages/automation/test/credential-fixtures'

vi.mock('#/pages/automation/ui/inline-credential-form', () => ({
	InlineCredentialForm: ({
		kinds,
		lockOrigin,
		error,
		onSubmit,
	}: {
		kinds: CredentialKind[]
		lockOrigin?: string
		error: Error | null
		onSubmit: (body: unknown) => void
	}) => (
		<div>
			<p>Types : {kinds.map((kind) => kind.kind).join(', ')}</p>
			<p>Origine : {lockOrigin ?? 'libre'}</p>
			{error ? <p>Échec : {error.message}</p> : null}
			<button
				type="button"
				onClick={() =>
					onSubmit({
						kind: kinds[0]?.kind,
						name: 'Ma clé',
						description: 'Clé du fournisseur',
						origin: lockOrigin ?? 'supplied',
					})
				}
			>
				Créer
			</button>
		</div>
	),
}))

for (const method of [
	'hasPointerCapture',
	'setPointerCapture',
	'releasePointerCapture',
	'scrollIntoView',
] as const) {
	if (typeof Element.prototype[method] !== 'function') {
		Element.prototype[method] = (() => false) as never
	}
}

const DESCRIPTOR: Schemas.ConnectorDescriptorResponse = {
	auth: { AnyOf: ['bearer_token', 'http_header'] },
	branches: [],
	execution: 'per_item',
	family: 'test',
	fields: [
		{
			name: 'signing_credential_id',
			label: 'Clé de signature sortante',
			kind: 'Text',
			expression: false,
			required: false,
			secret: false,
			visible_when: null,
		},
	],
	kind: 'test.fabricated',
	label: 'Fabriqué',
	output_example: null,
	version: 1,
}

function created(
	id: string,
	overrides: Partial<Schemas.CredentialResponse> = {},
): Schemas.CredentialResponse & { secret: unknown } {
	return { ...credential({ id, name: id, ...overrides }), secret: null }
}

function renderFeature(
	overrides: Partial<ConnectorParametersFeatureProps> = {},
) {
	const props: ConnectorParametersFeatureProps = {
		descriptor: DESCRIPTOR,
		config: {},
		credentialId: null,
		credentials: [],
		credentialKinds: [BEARER_TOKEN, HTTP_BASIC, HTTP_HEADER, ODOO_API],
		errors: [],
		bindingIssues: [],
		evaluateContext: { trigger: null, nodes: {}, position: 0, loop: null },
		onConfigChange: vi.fn(),
		onCredentialChange: vi.fn(),
		onCreateCredential: vi.fn(),
		onEvaluateExpression: vi.fn(),
		...overrides,
	}
	render(<ConnectorParametersFeature {...props} />)
	return props
}

function creationButtons() {
	return screen.getAllByRole('button', { name: /Créer une identification/ })
}

function typedCreation() {
	return creationButtons()[0] as HTMLElement
}

function signingCreation() {
	return creationButtons()[1] as HTMLElement
}

const SMTP_DESCRIPTOR: Schemas.ConnectorDescriptorResponse = {
	...DESCRIPTOR,
	auth: { Exactly: 'smtp' },
	family: 'smtp',
	fields: [],
	kind: 'smtp.send',
	label: 'E-mail SMTP',
}

describe('ConnectorParametersFeature — creating an identification from the node', () => {
	it('offers only the smtp kind to the e-mail node', async () => {
		const user = userEvent.setup()
		renderFeature({
			descriptor: SMTP_DESCRIPTOR,
			credentialKinds: [BEARER_TOKEN, SMTP, ODOO_API],
		})

		await user.click(typedCreation())

		expect(screen.getByText('Types : smtp')).toBeDefined()
	})

	it('offers only the kinds the connector accepts', async () => {
		const user = userEvent.setup()
		renderFeature()

		await user.click(typedCreation())

		expect(screen.getByText('Types : bearer_token, http_header')).toBeDefined()
		expect(screen.getByText('Origine : libre')).toBeDefined()
	})

	it('offers only the kinds Mestier can generate for the key that signs the calls', async () => {
		const user = userEvent.setup()
		renderFeature()

		await user.click(signingCreation())

		expect(screen.getByText('Types : bearer_token')).toBeDefined()
		expect(screen.getByText('Origine : generated')).toBeDefined()
	})

	it('hands the body, description included, to the creation and selects what comes back', async () => {
		const user = userEvent.setup()
		const props = renderFeature({
			onCreateCredential: vi.fn().mockResolvedValue(created('new-cred')),
		})

		await user.click(typedCreation())
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			expect(props.onCredentialChange).toHaveBeenCalledWith('new-cred')
		})
		expect(props.onCreateCredential).toHaveBeenCalledWith({
			kind: 'bearer_token',
			name: 'Ma clé',
			description: 'Clé du fournisseur',
			origin: 'supplied',
		})
	})

	it('shows the failure of the server and keeps the form', async () => {
		const user = userEvent.setup()
		renderFeature({
			onCreateCredential: vi
				.fn()
				.mockRejectedValue(
					Object.assign(new Error('Conflict: nom pris'), { status: 409 }),
				),
		})

		await user.click(typedCreation())
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(await screen.findByText('Échec : Conflict: nom pris')).toBeDefined()
		expect(screen.getByRole('button', { name: 'Créer' })).toBeDefined()
	})
})
