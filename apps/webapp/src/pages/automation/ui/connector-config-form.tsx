import type { ReactNode } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	isSigningCredentialField,
	visibleFields,
	withField,
	withoutField,
} from '#/pages/automation/lib/connector-config'
import {
	credentialSlotAfter,
	displayField,
} from '#/pages/automation/lib/connector-field-copy'
import type { CredentialStatus } from '#/pages/automation/lib/credential-meta'
import {
	credentialsForAuth,
	generatedCredentials,
	isCredentialRequired,
} from '#/pages/automation/lib/credential-picker'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import {
	connectorLevelErrors,
	fieldErrorMessage,
} from '#/pages/automation/lib/validation'
import { ConnectorConfigField } from '#/pages/automation/ui/connector-config-field'
import { CredentialPickerField } from '#/pages/automation/ui/credential-picker-field'

export type CredentialCreationPurpose = 'typed' | 'signing'

export interface ConnectorConfigFormProps {
	descriptor: Schemas.ConnectorDescriptorResponse
	config: Record<string, unknown>
	credentialId: string | null
	credentials: Schemas.CredentialResponse[]
	errors: ConnectorValidationError[]
	bindingIssues?: readonly NodeIssue[]
	credentialHint?: string
	credentialStatusOf?: (credentialId: string) => CredentialStatus
	renderPreview?: (field: Schemas.FieldResponse) => ReactNode
	onConfigChange: (config: Record<string, unknown>) => void
	onCredentialChange: (credentialId: string | null) => void
	onRequestCreateCredential: (purpose: CredentialCreationPurpose) => void
}

export function ConnectorConfigForm({
	descriptor,
	config,
	credentialId,
	credentials,
	errors,
	bindingIssues = [],
	credentialHint,
	credentialStatusOf,
	renderPreview,
	onConfigChange,
	onCredentialChange,
	onRequestCreateCredential,
}: ConnectorConfigFormProps) {
	const levelErrors = connectorLevelErrors(errors)

	function issuesOf(name: string): string[] {
		const server = fieldErrorMessage(errors, name)
		return [
			...(server ? [server] : []),
			...bindingIssues
				.filter((issue) => issue.parameter === name)
				.map((issue) => issue.message),
		]
	}

	function setField(name: string, value: unknown) {
		onConfigChange(
			value === undefined
				? withoutField(config, name)
				: withField(config, name, value),
		)
	}

	const credentialSlot =
		descriptor.auth !== 'None' ? (
			<CredentialPickerField
				key="credential-slot"
				label="Identification"
				htmlFor="connector-credential"
				credentials={credentialsForAuth(credentials, descriptor.auth)}
				value={credentialId}
				error={levelErrors[0] ?? null}
				optional={!isCredentialRequired(descriptor.auth)}
				hint={credentialHint}
				statusOf={credentialStatusOf}
				onChange={onCredentialChange}
				onCreateNew={() => onRequestCreateCredential('typed')}
			/>
		) : null
	const slotAfter = credentialSlotAfter(descriptor.kind)
	const fields = visibleFields(descriptor.fields, config)
	const slotFollowsAField = fields.some((field) => field.name === slotAfter)

	return (
		<div className="grid gap-4">
			{slotFollowsAField ? null : credentialSlot}

			{fields.flatMap((field) => {
				const shown = displayField(descriptor.kind, field)
				const current = config[field.name]
				const control = isSigningCredentialField(field) ? (
					<CredentialPickerField
						key={field.name}
						label={shown.label}
						htmlFor={`connector-field-${field.name}`}
						credentials={generatedCredentials(credentials)}
						value={typeof current === 'string' ? current : null}
						error={fieldErrorMessage(errors, field.name)}
						optional
						onChange={(next) => setField(field.name, next ?? undefined)}
						onCreateNew={() => onRequestCreateCredential('signing')}
					/>
				) : (
					<ConnectorConfigField
						key={field.name}
						field={shown}
						value={current}
						issues={issuesOf(field.name)}
						preview={renderPreview?.(field)}
						onChange={(value) => setField(field.name, value)}
					/>
				)
				return field.name === slotAfter ? [control, credentialSlot] : [control]
			})}
		</div>
	)
}
