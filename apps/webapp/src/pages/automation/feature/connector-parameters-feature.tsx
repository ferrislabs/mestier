import { useEffect, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	useCredentialMeta,
	useCredentialMetaActions,
} from '#/pages/automation/feature/use-credential-meta'
import { withField } from '#/pages/automation/lib/connector-config'
import { withFieldDefaults } from '#/pages/automation/lib/connector-field-copy'
import type { CredentialStatus } from '#/pages/automation/lib/credential-meta'
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

interface CreateState {
	purpose: CredentialCreationPurpose
	result: CreatedCredential | null
}

export interface ConnectorParametersFeatureProps {
	descriptor: Schemas.ConnectorDescriptorResponse
	config: Record<string, unknown>
	credentialId: string | null
	credentials: Schemas.CredentialResponse[]
	authSchemes: Schemas.AuthSchemeResponse[]
	errors: ConnectorValidationError[]
	bindingIssues: readonly NodeIssue[]
	credentialHint?: string
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
	authSchemes,
	errors,
	bindingIssues,
	credentialHint,
	evaluateContext,
	onConfigChange,
	onCredentialChange,
	onCreateCredential,
	onEvaluateExpression,
}: ConnectorParametersFeatureProps) {
	const [createState, setCreateState] = useState<CreateState | null>(null)
	const [createPending, setCreatePending] = useState(false)
	const [createError, setCreateError] = useState<string | null>(null)
	const configRef = useRef(config)
	configRef.current = config
	const changeRef = useRef(onConfigChange)
	changeRef.current = onConfigChange
	const credentialMeta = useCredentialMeta().data
	const credentialMetaActions = useCredentialMetaActions()
	const { rememberKinds } = credentialMetaActions

	useEffect(() => {
		rememberKinds(credentials)
	}, [credentials, rememberKinds])

	useEffect(() => {
		const seeded = withFieldDefaults(descriptor.kind, configRef.current)
		if (seeded) changeRef.current(seeded)
	}, [descriptor.kind])

	const statusOf = (credentialId: string): CredentialStatus =>
		credentialMeta[credentialId]?.status ?? 'valid'

	function applySelection(purpose: CredentialCreationPurpose, id: string) {
		if (purpose === 'typed') {
			onCredentialChange(id)
			return
		}
		onConfigChange(withField(config, 'signing_credential_id', id))
	}

	async function handleCreateSubmit(
		body: Schemas.CreateCredentialRequest,
		description?: string,
	) {
		if (!createState) return
		setCreatePending(true)
		setCreateError(null)
		try {
			const created = await onCreateCredential(body)
			const note = description?.trim()
			if (note) credentialMetaActions.patch(created.id, { description: note })
			if (created.origin === 'generated') {
				setCreateState({ purpose: createState.purpose, result: created })
			} else {
				applySelection(createState.purpose, created.id)
				setCreateState(null)
			}
		} catch (error) {
			setCreateError(
				error instanceof Error ? error.message : 'La création a échoué.',
			)
		} finally {
			setCreatePending(false)
		}
	}

	function handleCreateDone() {
		if (createState?.result) {
			applySelection(createState.purpose, createState.result.id)
		}
		setCreateState(null)
	}

	function handleCreateCancel() {
		setCreateState(null)
		setCreateError(null)
	}

	if (createState) {
		return createState.result ? (
			<CredentialSecretReveal
				secret={String(createState.result.secret)}
				onDone={handleCreateDone}
			/>
		) : (
			<InlineCredentialForm
				authSchemes={authSchemes}
				lockOrigin={createState.purpose === 'signing' ? 'generated' : undefined}
				isPending={createPending}
				error={createError}
				onSubmit={(body, description?: string) =>
					void handleCreateSubmit(body, description)
				}
				onCancel={handleCreateCancel}
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
				credentialStatusOf={statusOf}
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
				onRequestCreateCredential={(purpose) => {
					setCreateError(null)
					setCreateState({ purpose, result: null })
				}}
			/>
		</>
	)
}
