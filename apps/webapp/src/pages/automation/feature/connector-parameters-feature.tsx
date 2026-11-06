import { useEffect, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import type { CredentialKind } from '#/hooks/use-automation'
import {
	type FieldAddOns,
	withField,
} from '#/pages/automation/lib/connector-config'
import { withFieldDefaults } from '#/pages/automation/lib/connector-field-copy'
import { allowsGeneratedOrigin } from '#/pages/automation/lib/credential-kinds'
import { allowedCredentialKinds } from '#/pages/automation/lib/credential-picker'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'
import { isExpression } from '#/pages/automation/lib/parameters'
import {
	type ConnectorValidationError,
	connectorLevelErrors,
} from '#/pages/automation/lib/validation'
import {
	ConnectorConfigForm,
	type CredentialCreationPurpose,
} from '#/pages/automation/ui/connector-config-form'
import { CredentialSecretReveal } from '#/pages/automation/ui/credential-secret-reveal'
import {
	ExpressionPreview,
	type ExpressionPreviewState,
} from '#/pages/automation/ui/expression-preview'
import { InlineCredentialForm } from '#/pages/automation/ui/inline-credential-form'

const PREVIEW_DEBOUNCE_MS = 400

export type CreatedCredential = Schemas.CredentialResponse & { secret: unknown }

type CreationFailure = Error & { status?: number }

interface CreateState {
	purpose: CredentialCreationPurpose
	result: CreatedCredential | null
}

export interface CredentialCreation {
	state: CreateState | null
	pending: boolean
	error: CreationFailure | null
	start: (purpose: CredentialCreationPurpose) => void
	submit: (body: Schemas.CreateCredentialRequest) => void
	done: () => void
	cancel: () => void
}

export function useCredentialCreation({
	onCreateCredential,
	onSelect,
}: {
	onCreateCredential: (
		body: Schemas.CreateCredentialRequest,
	) => Promise<CreatedCredential>
	onSelect: (purpose: CredentialCreationPurpose, id: string) => void
}): CredentialCreation {
	const [state, setState] = useState<CreateState | null>(null)
	const [pending, setPending] = useState(false)
	const [error, setError] = useState<CreationFailure | null>(null)

	async function submitAsync(body: Schemas.CreateCredentialRequest) {
		if (!state) return
		setPending(true)
		setError(null)
		try {
			const created = await onCreateCredential(body)
			if (created.origin === 'generated') {
				setState({ purpose: state.purpose, result: created })
			} else {
				onSelect(state.purpose, created.id)
				setState(null)
			}
		} catch (failure) {
			setError(
				failure instanceof Error ? failure : new Error('La création a échoué.'),
			)
		} finally {
			setPending(false)
		}
	}

	return {
		state,
		pending,
		error,
		start: (purpose) => {
			setError(null)
			setState({ purpose, result: null })
		},
		submit: (body) => void submitAsync(body),
		done: () => {
			if (state?.result) onSelect(state.purpose, state.result.id)
			setState(null)
		},
		cancel: () => {
			setState(null)
			setError(null)
		},
	}
}

export function CredentialCreationPanel({
	creation,
	kinds,
	signingKinds = [],
}: {
	creation: CredentialCreation
	kinds: CredentialKind[]
	signingKinds?: CredentialKind[]
}) {
	const { state } = creation
	if (!state) return null
	return state.result ? (
		<CredentialSecretReveal
			secret={String(state.result.secret)}
			onDone={creation.done}
		/>
	) : (
		<InlineCredentialForm
			key={state.purpose}
			kinds={state.purpose === 'signing' ? signingKinds : kinds}
			lockOrigin={state.purpose === 'signing' ? 'generated' : undefined}
			isPending={creation.pending}
			error={creation.error}
			onSubmit={creation.submit}
			onCancel={creation.cancel}
		/>
	)
}

export interface ConnectorParametersFeatureProps {
	descriptor: Schemas.ConnectorDescriptorResponse
	config: Record<string, unknown>
	credentialId: string | null
	credentials: Schemas.CredentialResponse[]
	credentialKinds: CredentialKind[]
	errors: ConnectorValidationError[]
	bindingIssues: readonly NodeIssue[]
	credentialHint?: string
	fieldAddOns?: Readonly<Record<string, FieldAddOns>>
	evaluateContext: Schemas.EvaluateContextBody
	onConfigChange: (config: Record<string, unknown>) => void
	onCredentialChange: (credentialId: string | null) => void
	onCreateCredential: (
		body: Schemas.CreateCredentialRequest,
	) => Promise<CreatedCredential>
	onEvaluateExpression: (
		template: unknown,
		context: Schemas.EvaluateContextBody,
	) => Promise<unknown>
}

function FieldPreview({
	template,
	context,
	onEvaluate,
}: {
	template: string
	context: Schemas.EvaluateContextBody
	onEvaluate: ConnectorParametersFeatureProps['onEvaluateExpression']
}) {
	const [state, setState] = useState<ExpressionPreviewState>({
		status: 'loading',
	})
	const evaluateRef = useRef(onEvaluate)
	evaluateRef.current = onEvaluate

	useEffect(() => {
		let cancelled = false
		setState({ status: 'loading' })
		const timer = setTimeout(() => {
			evaluateRef
				.current(template, context)
				.then((value) => {
					if (!cancelled) setState({ status: 'resolved', value })
				})
				.catch((error: unknown) => {
					if (!cancelled) {
						setState({
							status: 'error',
							message:
								error instanceof Error
									? error.message
									: 'L’évaluation a échoué.',
						})
					}
				})
		}, PREVIEW_DEBOUNCE_MS)

		return () => {
			cancelled = true
			clearTimeout(timer)
		}
	}, [template, context])

	return <ExpressionPreview state={state} />
}

export function ConnectorParametersFeature({
	descriptor,
	config,
	credentialId,
	credentials,
	credentialKinds,
	errors,
	bindingIssues,
	credentialHint,
	fieldAddOns,
	evaluateContext,
	onConfigChange,
	onCredentialChange,
	onCreateCredential,
	onEvaluateExpression,
}: ConnectorParametersFeatureProps) {
	const configRef = useRef(config)
	configRef.current = config
	const changeRef = useRef(onConfigChange)
	changeRef.current = onConfigChange
	useEffect(() => {
		const seeded = withFieldDefaults(descriptor.kind, configRef.current)
		if (seeded) changeRef.current(seeded)
	}, [descriptor.kind])

	const allowedKinds = allowedCredentialKinds(descriptor.auth)
	const typedKinds = credentialKinds.filter((candidate) =>
		allowedKinds.includes(candidate.kind),
	)

	const creation = useCredentialCreation({
		onCreateCredential,
		onSelect: (purpose, id) => {
			if (purpose === 'typed') {
				onCredentialChange(id)
				return
			}
			onConfigChange(withField(config, 'signing_credential_id', id))
		},
	})

	if (creation.state) {
		return (
			<CredentialCreationPanel
				creation={creation}
				kinds={typedKinds}
				signingKinds={credentialKinds.filter(allowsGeneratedOrigin)}
			/>
		)
	}

	const levelErrors = connectorLevelErrors(errors)

	return (
		<>
			{descriptor.auth === 'None' && levelErrors.length > 0 ? (
				<div
					role="alert"
					className="rounded-md border border-state-alert/60 bg-state-alert/10 px-3 py-2 text-sm text-state-alert"
				>
					<ul>
						{levelErrors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				</div>
			) : null}
			<ConnectorConfigForm
				descriptor={descriptor}
				config={config}
				credentialId={credentialId}
				credentials={credentials}
				errors={errors}
				bindingIssues={bindingIssues}
				credentialHint={credentialHint}
				fieldAddOns={fieldAddOns}
				renderPreview={(field) => {
					const template = config[field.name]
					return typeof template === 'string' && isExpression(template) ? (
						<FieldPreview
							template={template}
							context={evaluateContext}
							onEvaluate={onEvaluateExpression}
						/>
					) : null
				}}
				onConfigChange={onConfigChange}
				onCredentialChange={onCredentialChange}
				onRequestCreateCredential={creation.start}
			/>
		</>
	)
}
