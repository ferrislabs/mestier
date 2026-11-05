import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { ConnectorParametersFeature } from '#/pages/automation/feature/connector-parameters-feature'
import {
	forgetCredentialMeta,
	readCredentialMeta,
} from '#/pages/automation/feature/mocks/credential-meta'

vi.mock('#/pages/automation/ui/inline-credential-form', () => ({
	InlineCredentialForm: ({
		onSubmit,
	}: {
		onSubmit: (body: unknown, description?: string) => void
	}) => (
		<div>
			<button
				type="button"
				onClick={() =>
					onSubmit(
						{ kind: 'scheme_a', name: 'Ma clé', origin: 'supplied' },
						'  Clé du fournisseur  ',
					)
				}
			>
				Créer avec une description
			</button>
			<button
				type="button"
				onClick={() =>
					onSubmit({ kind: 'scheme_a', name: 'Sans', origin: 'supplied' })
				}
			>
				Créer sans description
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
	auth: { Exactly: 'scheme_a' },
	branches: [],
	family: 'test',
	fields: [],
	kind: 'test.fabricated',
	label: 'Fabriqué',
	output_example: null,
	version: 1,
}

function created(id: string): Schemas.CredentialResponse & { secret: unknown } {
	return {
		id,
		kind: 'scheme_a',
		name: id,
		origin: 'supplied',
		organization_id: 'org-1',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		secret: null,
	}
}

async function renderOpened(onCreateCredential: ReturnType<typeof vi.fn>) {
	const user = userEvent.setup()
	render(
		<ConnectorParametersFeature
			descriptor={DESCRIPTOR}
			config={{}}
			credentialId={null}
			credentials={[]}
			authSchemes={[]}
			errors={[]}
			bindingIssues={[]}
			evaluateContext={{ trigger: null, connectors: {}, loop: null }}
			onConfigChange={vi.fn()}
			onCredentialChange={vi.fn()}
			onCreateCredential={onCreateCredential}
			onEvaluateExpression={vi.fn()}
		/>,
	)
	await user.click(
		screen.getByRole('button', { name: /Créer une identification/ }),
	)
	return user
}

describe('ConnectorParametersFeature — the description of an inline identification', () => {
	afterEach(() => {
		forgetCredentialMeta('with-note')
		forgetCredentialMeta('without-note')
	})

	it('keeps the description the inline form hands over, trimmed', async () => {
		const onCreateCredential = vi.fn().mockResolvedValue(created('with-note'))
		const user = await renderOpened(onCreateCredential)

		await user.click(
			screen.getByRole('button', { name: 'Créer avec une description' }),
		)

		await waitFor(() => {
			expect(readCredentialMeta()['with-note']?.description).toBe(
				'Clé du fournisseur',
			)
		})
		expect(onCreateCredential).toHaveBeenCalledWith({
			kind: 'scheme_a',
			name: 'Ma clé',
			origin: 'supplied',
		})
	})

	it('writes no description when the form hands none over', async () => {
		const onCreateCredential = vi
			.fn()
			.mockResolvedValue(created('without-note'))
		const user = await renderOpened(onCreateCredential)

		await user.click(
			screen.getByRole('button', { name: 'Créer sans description' }),
		)

		await waitFor(() => {
			expect(onCreateCredential).toHaveBeenCalled()
		})
		expect(readCredentialMeta()['without-note']?.description).toBe('')
	})
})
