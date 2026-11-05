import { Loader2 } from 'lucide-react'
import { useId, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	buildCredentialFieldErrors,
	credentialFormDefaults,
	credentialPayload,
	translateCredentialMessage,
	visibleCredentialFields,
} from '#/pages/automation/lib/credential-form'
import { localizeScheme } from '#/pages/automation/lib/credential-meta'
import { CredentialFieldRows } from '#/pages/automation/ui/credential-dialog'

type Origin = 'supplied' | 'generated'

export interface InlineCredentialFormProps {
	authSchemes: Schemas.AuthSchemeResponse[]
	lockOrigin?: 'generated'
	isPending: boolean
	error: string | null
	onSubmit: (body: Schemas.CreateCredentialRequest) => void
	onCancel: () => void
}

export function InlineCredentialForm({
	authSchemes,
	lockOrigin,
	isPending,
	error,
	onSubmit,
	onCancel,
}: InlineCredentialFormProps) {
	const baseId = useId()
	const schemes = authSchemes.map(localizeScheme)
	const [name, setName] = useState('')
	const [kind, setKind] = useState(schemes[0]?.kind ?? '')
	const [origin, setOrigin] = useState<Origin>(lockOrigin ?? 'supplied')
	const [data, setData] = useState<Record<string, string>>(() =>
		credentialFormDefaults(schemes[0]),
	)
	const [showErrors, setShowErrors] = useState(false)

	const scheme = schemes.find((candidate) => candidate.kind === kind)
	const fieldErrors = buildCredentialFieldErrors(
		{ kind, name, description: '', origin, data },
		scheme,
		'create',
	)
	const errorOf = (key: string) => (showErrors ? fieldErrors[key] : undefined)
	const nameError = errorOf('name')

	function pickKind(next: string) {
		setKind(next)
		setData(
			credentialFormDefaults(
				schemes.find((candidate) => candidate.kind === next),
			),
		)
	}

	function handleSubmit() {
		if (isPending) return
		if (kind === '' || Object.keys(fieldErrors).length > 0) {
			setShowErrors(true)
			return
		}
		onSubmit({
			kind,
			name: name.trim(),
			origin,
			data:
				origin === 'supplied'
					? scheme
						? credentialPayload(scheme, data)
						: data
					: undefined,
		})
	}

	return (
		<div className="grid gap-4">
			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-name`}>Nom</Label>
				<Input
					id={`${baseId}-name`}
					value={name}
					onChange={(event) => setName(event.target.value)}
					placeholder={scheme ? `Ex. ${scheme.label} – fournisseur` : undefined}
					aria-invalid={nameError ? true : undefined}
					aria-describedby={nameError ? `${baseId}-name-error` : undefined}
				/>
				{nameError ? (
					<p id={`${baseId}-name-error`} className="text-state-alert text-xs">
						{nameError}
					</p>
				) : null}
			</div>

			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-kind`}>Type</Label>
				<Select value={kind} onValueChange={pickKind}>
					<SelectTrigger id={`${baseId}-kind`} className="w-full">
						<SelectValue placeholder="Choisir un type" />
					</SelectTrigger>
					<SelectContent>
						{schemes.map((candidate) => (
							<SelectItem key={candidate.kind} value={candidate.kind}>
								{candidate.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>

			{lockOrigin ? null : (
				<div className="grid gap-2">
					<Label htmlFor={`${baseId}-origin`}>Origine</Label>
					<Select
						value={origin}
						onValueChange={(next) => setOrigin(next as Origin)}
					>
						<SelectTrigger id={`${baseId}-origin`} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="supplied">
								Fournie par vous (mot de passe, clé d’API…)
							</SelectItem>
							<SelectItem value="generated">
								Générée par Mestier (secret de signature)
							</SelectItem>
						</SelectContent>
					</Select>
				</div>
			)}

			{origin === 'supplied' && scheme ? (
				<CredentialFieldRows
					fields={visibleCredentialFields(scheme.fields, data)}
					data={data}
					baseId={baseId}
					stored={false}
					errorOf={errorOf}
					onChange={setData}
				/>
			) : null}

			{error ? (
				<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{translateCredentialMessage(error, scheme?.fields)}
				</p>
			) : null}

			<div className="flex justify-end gap-2">
				<Button type="button" variant="outline" onClick={onCancel}>
					Annuler
				</Button>
				<Button type="button" disabled={isPending} onClick={handleSubmit}>
					{isPending ? <Loader2 className="animate-spin" /> : null}
					Créer
				</Button>
			</div>
		</div>
	)
}
