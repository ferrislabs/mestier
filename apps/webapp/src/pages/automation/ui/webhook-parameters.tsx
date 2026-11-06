import { Unlink } from 'lucide-react'
import { useId } from 'react'
import type { Schemas } from '#/api/api.client'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { CredentialStatus } from '#/pages/automation/lib/credential-meta'
import type {
	ContractKey,
	WebhookRule,
} from '#/pages/automation/lib/webhook-contract'
import { CredentialPickerField } from '#/pages/automation/ui/credential-picker-field'
import { WebhookContractEditor } from '#/pages/automation/ui/webhook-contract-editor'
import { WebhookPathField } from '#/pages/automation/ui/webhook-path-field'

const HTTP_METHODS: readonly Schemas.WebhookMethodDto[] = [
	'GET',
	'POST',
	'PUT',
	'PATCH',
	'DELETE',
]

const INVALID_BODY_OPTIONS: readonly {
	value: Schemas.OnInvalidDto
	label: string
}[] = [
	{ value: 'Reject', label: 'Refuser (réponse 400)' },
	{ value: 'Accept', label: 'Accepter et lancer le workflow' },
]

export const NO_CREDENTIAL_WARNING =
	'Sans identification, toute personne qui connaît cette URL peut déclencher ce workflow.'

export const PATH_HINT =
	'Le préfixe /webhooks/ est imposé. La suite est libre : lettres minuscules, chiffres, « - », « _ » et « / ».'

export interface WebhookMessages {
	path: readonly string[]
	credential: string | null
	contract: readonly string[]
	keys: Readonly<Record<string, string>>
}

function FieldMessages({ messages }: { messages: readonly string[] }) {
	return messages.map((message) => (
		<p
			key={message}
			role="alert"
			className="flex items-start gap-1.5 text-state-alert text-xs"
		>
			<Unlink className="mt-0.5 size-3.5 shrink-0" aria-hidden />
			{message}
		</p>
	))
}

export function WebhookParameters({
	rule,
	origin,
	credentials,
	messages,
	credentialHint,
	credentialStatusOf,
	onChange,
	onCreateCredential,
}: {
	rule: WebhookRule
	origin: string
	credentials: Schemas.CredentialResponse[]
	messages: WebhookMessages
	credentialHint: string
	credentialStatusOf: (credentialId: string) => CredentialStatus
	onChange: (rule: WebhookRule) => void
	onCreateCredential: () => void
}) {
	const baseId = useId()
	const credentialId = rule.credential_id ?? null

	return (
		<>
			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-method`}>Méthode</Label>
				<Select
					value={rule.method}
					onValueChange={(method) =>
						onChange({ ...rule, method: method as Schemas.WebhookMethodDto })
					}
				>
					<SelectTrigger id={`${baseId}-method`} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{HTTP_METHODS.map((method) => (
							<SelectItem key={method} value={method}>
								{method}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-path`}>URL</Label>
				<WebhookPathField
					controlId={`${baseId}-path`}
					value={rule.path}
					invalid={messages.path.length > 0}
					origin={origin}
					onChange={(path) => onChange({ ...rule, path })}
				/>
				<FieldMessages messages={messages.path} />
				<p className="text-muted-foreground text-xs">{PATH_HINT}</p>
			</div>
			<CredentialPickerField
				label="Identification"
				htmlFor={`${baseId}-credential`}
				credentials={credentials}
				value={credentialId}
				error={messages.credential}
				optional
				hint={credentialHint}
				warning={credentialId === null ? NO_CREDENTIAL_WARNING : undefined}
				statusOf={credentialStatusOf}
				onChange={(credential_id) => onChange({ ...rule, credential_id })}
				onCreateNew={onCreateCredential}
			/>
			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-contract`}>Contrat du corps reçu</Label>
				<WebhookContractEditor
					controlId={`${baseId}-contract`}
					contract={rule.contract ?? []}
					keyErrors={messages.keys}
					onChange={(contract: ContractKey[]) =>
						onChange({ ...rule, contract })
					}
				/>
				<FieldMessages messages={messages.contract} />
				<p className="text-muted-foreground text-xs">
					Les nœuds suivants utilisent ces clés typées.
				</p>
			</div>
			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-invalid`}>
					Requête non conforme au contrat
				</Label>
				<Select
					value={rule.on_invalid ?? 'Reject'}
					onValueChange={(on_invalid) =>
						onChange({
							...rule,
							on_invalid: on_invalid as Schemas.OnInvalidDto,
						})
					}
				>
					<SelectTrigger id={`${baseId}-invalid`} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{INVALID_BODY_OPTIONS.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
		</>
	)
}
