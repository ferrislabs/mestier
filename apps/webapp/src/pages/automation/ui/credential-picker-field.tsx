import { Plus, TriangleAlert, Unlink } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { CredentialStatus } from '#/pages/automation/lib/credential-meta'

const NONE_VALUE = '__none__'

export interface CredentialPickerFieldProps {
	label: string
	htmlFor: string
	credentials: Schemas.CredentialResponse[]
	value: string | null
	error: string | null
	optional?: boolean
	hint?: string
	statusOf?: (credentialId: string) => CredentialStatus
	onChange: (credentialId: string | null) => void
	onCreateNew: () => void
}

export function CredentialPickerField({
	label,
	htmlFor,
	credentials,
	value,
	error,
	optional = false,
	hint,
	statusOf,
	onChange,
	onCreateNew,
}: CredentialPickerFieldProps) {
	const selected =
		credentials.find((credential) => credential.id === value) ?? null
	const isExpired = (credentialId: string) =>
		statusOf?.(credentialId) === 'expired'

	return (
		<div className="grid gap-2">
			<Label htmlFor={htmlFor}>{label}</Label>
			<Select
				value={selected?.id ?? ''}
				disabled={credentials.length === 0}
				onValueChange={(next) => onChange(next === NONE_VALUE ? null : next)}
			>
				<SelectTrigger
					id={htmlFor}
					className="w-full"
					aria-invalid={error !== null ? true : undefined}
				>
					<SelectValue
						placeholder={
							credentials.length === 0
								? 'Aucune identification'
								: 'Choisir une identification'
						}
					/>
				</SelectTrigger>
				<SelectContent>
					{optional ? (
						<SelectItem value={NONE_VALUE}>Aucune identification</SelectItem>
					) : null}
					{credentials.map((credential) => (
						<SelectItem key={credential.id} value={credential.id}>
							{credential.name}
							{isExpired(credential.id) ? ' (expirée)' : ''}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			{selected && isExpired(selected.id) ? (
				<p className="flex items-start gap-1.5 text-sm text-state-alert">
					<TriangleAlert className="mt-0.5 size-4 shrink-0" aria-hidden />
					Cette identification est expirée. Renouvelez-la depuis
					Identifications.
				</p>
			) : null}
			<Button
				type="button"
				variant="ghost"
				size="sm"
				className="-ml-2 justify-self-start text-muted-foreground"
				onClick={onCreateNew}
			>
				<Plus />
				Créer une identification
			</Button>
			{error ? (
				<p
					role="alert"
					className="flex items-start gap-1.5 text-state-alert text-xs"
				>
					<Unlink className="mt-0.5 size-3.5 shrink-0" aria-hidden />
					{error}
				</p>
			) : null}
			{hint ? <p className="text-muted-foreground text-xs">{hint}</p> : null}
		</div>
	)
}
