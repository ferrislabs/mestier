import { useEffect, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { withField } from '#/pages/automation/lib/connector-config'
import { insertExpressionAtCursor } from '#/pages/automation/lib/expression'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'
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

type FieldControlElement = HTMLInputElement | HTMLTextAreaElement

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

export function ConnectorParametersFeature({
	descriptor,
	config,
	credentialId,
	credentials,
	authSchemes,
	errors,
	bindingIssues,
	evaluateContext,
	onConfigChange,
	onCredentialChange,
	onCreateCredential,
	onEvaluateExpression,
}: ConnectorParametersFeatureProps) {
	const [createState, setCreateState] = useState<CreateState | null>(null)
	const [createPending, setCreatePending] = useState(false)
	const [createError, setCreateError] = useState<string | null>(null)
	const [activeField, setActiveField] = useState<Schemas.FieldResponse | null>(
		null,
	)
	const [previewState, setPreviewState] = useState<ExpressionPreviewState>({
		status: 'idle',
	})
	const fieldRefs = useRef(new Map<string, FieldControlElement>())
	const evaluateRef = useRef(onEvaluateExpression)
	evaluateRef.current = onEvaluateExpression

	useEffect(() => {
		if (!activeField) {
			setPreviewState({ status: 'idle' })
			return
		}

		const template = config[activeField.name]
		if (
			template === undefined ||
			(typeof template === 'string' && template.trim() === '')
		) {
			setPreviewState({ status: 'idle' })
			return
		}

		let cancelled = false
		setPreviewState({ status: 'loading' })
		const timer = setTimeout(() => {
			evaluateRef
				.current(template, evaluateContext)
				.then((value) => {
					if (!cancelled) setPreviewState({ status: 'resolved', value })
				})
				.catch((error: unknown) => {
					if (!cancelled) {
						setPreviewState({
							status: 'error',
							message:
								error instanceof Error
									? error.message
									: "L'évaluation a échoué.",
						})
					}
				})
		}, PREVIEW_DEBOUNCE_MS)

		return () => {
			cancelled = true
			clearTimeout(timer)
		}
	}, [activeField, config, evaluateContext])

	function handleFieldDrop(fieldName: string, path: string) {
		const field = descriptor.fields.find(
			(candidate) => candidate.name === fieldName,
		)
		if (field) setActiveField(field)

		const currentValue = config[fieldName]
		const text = typeof currentValue === 'string' ? currentValue : ''
		const element = fieldRefs.current.get(fieldName)
		const selection = {
			start: element?.selectionStart ?? text.length,
			end: element?.selectionEnd ?? text.length,
		}

		const result = insertExpressionAtCursor(text, path, selection)
		onConfigChange(withField(config, fieldName, result.text))

		if (element) {
			element.focus()
			element.setSelectionRange(result.cursor, result.cursor)
		}
	}

	function applySelection(purpose: CredentialCreationPurpose, id: string) {
		if (purpose === 'typed') {
			onCredentialChange(id)
			return
		}
		onConfigChange(withField(config, 'signing_credential_id', id))
	}

	async function handleCreateSubmit(body: Schemas.CreateCredentialRequest) {
		if (!createState) return
		setCreatePending(true)
		setCreateError(null)
		try {
			const created = await onCreateCredential(body)
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
				onSubmit={(body) => void handleCreateSubmit(body)}
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
				activeFieldName={activeField?.name ?? null}
				preview={<ExpressionPreview state={previewState} />}
				onConfigChange={onConfigChange}
				onCredentialChange={onCredentialChange}
				onActivateField={setActiveField}
				onFieldRef={(name, element) => {
					if (element) fieldRefs.current.set(name, element)
					else fieldRefs.current.delete(name)
				}}
				onInsertExpression={handleFieldDrop}
				onRequestCreateCredential={(purpose) => {
					setCreateError(null)
					setCreateState({ purpose, result: null })
				}}
			/>
		</>
	)
}
