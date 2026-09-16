import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { isSelectKind } from '#/pages/automation/lib/connector-config'

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
	const [name, setName] = useState('')
	const [kind, setKind] = useState(authSchemes[0]?.kind ?? '')
	const [origin, setOrigin] = useState<Origin>(lockOrigin ?? 'supplied')
	const [data, setData] = useState<Record<string, string>>({})

	const scheme = authSchemes.find((candidate) => candidate.kind === kind)
	const showData = origin === 'supplied' && scheme !== undefined
	const missingRequiredField =
		showData &&
		scheme.fields.some(
			(field) => field.required && (data[field.name] ?? '').trim() === '',
		)
	const canSubmit =
		!isPending && name.trim() !== '' && kind !== '' && !missingRequiredField

	function handleSubmit() {
		if (!canSubmit) return
		onSubmit({
			kind,
			name: name.trim(),
			origin,
			data: origin === 'supplied' ? data : undefined,
		})
	}

	return (
		<div className="flex flex-col gap-4">
			<Field label="Nom" htmlFor="inline-credential-name">
				<Input
					id="inline-credential-name"
					value={name}
					onChange={(event) => setName(event.target.value)}
				/>
			</Field>

			<Field label="Type" htmlFor="inline-credential-kind">
				<Select value={kind} onValueChange={setKind}>
					<SelectTrigger id="inline-credential-kind" className="w-full">
						<SelectValue placeholder="Choisir un type" />
					</SelectTrigger>
					<SelectContent>
						{authSchemes.map((candidate) => (
							<SelectItem key={candidate.kind} value={candidate.kind}>
								{candidate.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</Field>

			{lockOrigin ? null : (
				<Field label="Origine" htmlFor="inline-credential-origin">
					<Select
						value={origin}
						onValueChange={(next) => setOrigin(next as Origin)}
					>
						<SelectTrigger id="inline-credential-origin" className="w-full">
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
				</Field>
			)}

			{showData
				? scheme.fields.map((field) => (
						<CredentialDataField
							key={field.name}
							field={field}
							value={data[field.name] ?? ''}
							onChange={(value) => setData({ ...data, [field.name]: value })}
						/>
					))
				: null}

			{error ? <p className="text-sm text-destructive">{error}</p> : null}

			<div className="flex justify-end gap-2">
				<Button type="button" variant="ghost" onClick={onCancel}>
					Annuler
				</Button>
				<Button type="button" disabled={!canSubmit} onClick={handleSubmit}>
					{isPending ? <Loader2 className="animate-spin" /> : null}
					Créer
				</Button>
			</div>
		</div>
	)
}

function CredentialDataField({
	field,
	value,
	onChange,
}: {
	field: Schemas.FieldResponse
	value: string
	onChange: (value: string) => void
}) {
	const id = `inline-credential-field-${field.name}`

	if (isSelectKind(field.kind)) {
		return (
			<Field label={field.label} htmlFor={id}>
				<Select value={value} onValueChange={onChange}>
					<SelectTrigger id={id} className="w-full">
						<SelectValue placeholder="Choisir…" />
					</SelectTrigger>
					<SelectContent>
						{field.kind.Select.options.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</Field>
		)
	}

	return (
		<Field label={field.label} htmlFor={id}>
			<Input
				id={id}
				type={
					field.secret
						? 'password'
						: field.kind === 'Number'
							? 'number'
							: 'text'
				}
				value={value}
				autoComplete="off"
				onChange={(event) => onChange(event.target.value)}
			/>
		</Field>
	)
}
