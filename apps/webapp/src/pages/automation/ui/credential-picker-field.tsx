import { Plus } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Field } from '#/components/ui/field'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'

export interface CredentialPickerFieldProps {
	label: string
	htmlFor: string
	credentials: Schemas.CredentialResponse[]
	value: string | null
	error: string | null
	onChange: (credentialId: string | null) => void
	onCreateNew: () => void
}

export function CredentialPickerField({
	label,
	htmlFor,
	credentials,
	value,
	error,
	onChange,
	onCreateNew,
}: CredentialPickerFieldProps) {
	return (
		<Field label={label} htmlFor={htmlFor}>
			<div className="flex items-center gap-2">
				<Select
					value={value ?? undefined}
					onValueChange={(next) => onChange(next)}
				>
					<SelectTrigger
						id={htmlFor}
						className="w-full"
						aria-invalid={error !== null}
					>
						<SelectValue
							placeholder={
								credentials.length === 0
									? 'Aucune identification'
									: 'Choisir une identification…'
							}
						/>
					</SelectTrigger>
					<SelectContent>
						{credentials.length === 0 ? (
							<p className="px-2 py-4 text-center text-sm text-muted-foreground">
								Aucune identification disponible
							</p>
						) : (
							credentials.map((credential) => (
								<SelectItem key={credential.id} value={credential.id}>
									{credential.name}
								</SelectItem>
							))
						)}
					</SelectContent>
				</Select>
				<Button
					type="button"
					variant={credentials.length === 0 ? 'default' : 'outline'}
					size={credentials.length === 0 ? 'sm' : 'icon'}
					aria-label={
						credentials.length === 0 ? undefined : 'Nouvelle identification'
					}
					title="Nouvelle identification"
					onClick={onCreateNew}
				>
					<Plus className="size-4" />
					{credentials.length === 0 ? 'Créer' : null}
				</Button>
			</div>
			{error ? (
				<p role="alert" className="text-sm text-destructive">
					{error}
				</p>
			) : null}
		</Field>
	)
}
