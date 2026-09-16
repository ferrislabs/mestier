import type { Schemas } from '#/api/api.client'
import {
	isSigningCredentialField,
	visibleFields,
	withField,
	withoutField,
} from '#/pages/automation/lib/connector-config'
import {
	credentialsForAuth,
	generatedCredentials,
} from '#/pages/automation/lib/credential-picker'
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
	onConfigChange: (config: Record<string, unknown>) => void
	onCredentialChange: (credentialId: string | null) => void
	onOpenExpression: (field: Schemas.FieldResponse) => void
	onRequestCreateCredential: (purpose: CredentialCreationPurpose) => void
}

export function ConnectorConfigForm({
	descriptor,
	config,
	credentialId,
	credentials,
	errors,
	onConfigChange,
	onCredentialChange,
	onOpenExpression,
	onRequestCreateCredential,
}: ConnectorConfigFormProps) {
	const levelErrors = connectorLevelErrors(errors)

	function setField(name: string, value: unknown) {
		onConfigChange(
			value === undefined
				? withoutField(config, name)
				: withField(config, name, value),
		)
	}

	return (
		<div className="flex flex-col gap-5">
			{descriptor.auth !== 'None' ? (
				<CredentialPickerField
					label="Identification"
					htmlFor="connector-credential"
					credentials={credentialsForAuth(credentials, descriptor.auth)}
					value={credentialId}
					error={levelErrors[0] ?? null}
					onChange={onCredentialChange}
					onCreateNew={() => onRequestCreateCredential('typed')}
				/>
			) : null}

			{visibleFields(descriptor.fields, config).map((field) => {
				if (isSigningCredentialField(field)) {
					const value = config[field.name]
					return (
						<CredentialPickerField
							key={field.name}
							label={field.label}
							htmlFor={`connector-field-${field.name}`}
							credentials={generatedCredentials(credentials)}
							value={typeof value === 'string' ? value : null}
							error={fieldErrorMessage(errors, field.name)}
							onChange={(next) => setField(field.name, next ?? undefined)}
							onCreateNew={() => onRequestCreateCredential('signing')}
						/>
					)
				}

				return (
					<ConnectorConfigField
						key={field.name}
						field={field}
						value={config[field.name]}
						error={fieldErrorMessage(errors, field.name)}
						onChange={(value) => setField(field.name, value)}
						onOpenExpression={() => onOpenExpression(field)}
					/>
				)
			})}
		</div>
	)
}
