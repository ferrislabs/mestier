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
import type { CredentialKind } from '#/hooks/use-automation'
import { credentialFailure } from '#/pages/automation/lib/credential-errors'
import {
	buildCredentialFieldErrors,
	credentialFormDefaults,
	credentialPayload,
	visibleCredentialFields,
} from '#/pages/automation/lib/credential-form'
import { allowsGeneratedOrigin } from '#/pages/automation/lib/credential-kinds'
import { CredentialFieldRows } from '#/pages/automation/ui/credential-dialog'
import { TextArea } from '#/pages/automation/ui/text-area'

type Origin = 'supplied' | 'generated'

export interface InlineCredentialFormProps {
	kinds: CredentialKind[]
	lockOrigin?: 'generated'
	isPending: boolean
	error: (Error & { status?: number }) | null
	onSubmit: (body: Schemas.CreateCredentialRequest) => void
	onCancel: () => void
}

export function InlineCredentialForm({
	kinds,
	lockOrigin,
	isPending,
	error,
	onSubmit,
	onCancel,
}: InlineCredentialFormProps) {
	const baseId = useId()
	const [name, setName] = useState('')
	const [description, setDescription] = useState('')
	const [kindCode, setKindCode] = useState(kinds[0]?.kind ?? '')
	const [origin, setOrigin] = useState<Origin>(lockOrigin ?? 'supplied')
	const [data, setData] = useState<Record<string, string>>(() =>
		credentialFormDefaults(kinds[0]),
	)
	const [showErrors, setShowErrors] = useState(false)

	const kind = kinds.find((candidate) => candidate.kind === kindCode)
	const fieldErrors = buildCredentialFieldErrors(
		{ kind: kindCode, name, description, origin, data },
		kind,
	)
	const failure = credentialFailure(error, kind?.fields)
	const failureOf = (key: string) =>
		failure?.field === key ? failure.message : undefined
	const errorOf = (key: string) =>
		(showErrors ? fieldErrors[key] : undefined) ?? failureOf(key)
	const nameError = errorOf('name')

	function pickKind(next: string) {
		setKindCode(next)
		setOrigin(lockOrigin ?? 'supplied')
		setData(
			credentialFormDefaults(
				kinds.find((candidate) => candidate.kind === next),
			),
		)
	}

	function handleSubmit() {
		if (isPending) return
		if (!kind || Object.keys(fieldErrors).length > 0) {
			setShowErrors(true)
			return
		}
		const note = description.trim()
		onSubmit({
			kind: kind.kind,
			name: name.trim(),
			...(note === '' ? {} : { description: note }),
			origin,
			data: origin === 'supplied' ? credentialPayload(kind, data) : undefined,
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
					placeholder={kind ? `Ex. ${kind.label} – fournisseur` : undefined}
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
				<Label htmlFor={`${baseId}-description`}>
					Description{' '}
					<span className="font-normal text-muted-foreground">
						(facultatif)
					</span>
				</Label>
				<TextArea
					id={`${baseId}-description`}
					rows={2}
					value={description}
					onChange={(event) => setDescription(event.target.value)}
					placeholder="À quoi sert-elle ? Qui l’a fournie ?"
				/>
				{failureOf('description') ? (
					<p className="text-state-alert text-xs">{failureOf('description')}</p>
				) : null}
			</div>

			<div className="grid gap-2">
				<Label htmlFor={`${baseId}-kind`}>Type</Label>
				<Select value={kindCode} onValueChange={pickKind}>
					<SelectTrigger id={`${baseId}-kind`} className="w-full">
						<SelectValue placeholder="Choisir un type" />
					</SelectTrigger>
					<SelectContent>
						{kinds.map((candidate) => (
							<SelectItem key={candidate.kind} value={candidate.kind}>
								{candidate.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>

			{!lockOrigin && kind && allowsGeneratedOrigin(kind) ? (
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
			) : null}

			{origin === 'supplied' && kind ? (
				<CredentialFieldRows
					fields={visibleCredentialFields(kind.fields, data)}
					data={data}
					baseId={baseId}
					storedSecrets={[]}
					errorOf={errorOf}
					onChange={setData}
				/>
			) : null}

			{failure && failure.field === null ? (
				<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{failure.message}
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
