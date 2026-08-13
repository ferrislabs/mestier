import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'

export interface CredentialPickerOption {
	id: string
	name: string
}

export interface CredentialPickerProps {
	/** Already filtered by the feature layer against the connector's
	 * `AuthRequirement` — this component never sees a credential its scheme
	 * doesn't accept, so it never has to decide that itself. */
	options: CredentialPickerOption[]
	value: string | null
	onChange: (credentialId: string | null) => void
	/** Opens the inline creation sheet — rendered by the feature layer with
	 * `CredentialFormSheet` (from `#/pages/settings/ui/automation-credentials-ui`),
	 * not duplicated here. */
	onCreateNew: () => void
	disabled?: boolean
}

export function CredentialPicker({
	options,
	value,
	onChange,
	onCreateNew,
	disabled,
}: CredentialPickerProps) {
	return (
		<div className="flex gap-2">
			<Select
				value={value ?? ''}
				onValueChange={(next) => onChange(next === '' ? null : next)}
				disabled={disabled}
			>
				<SelectTrigger className="w-full">
					<SelectValue placeholder="Aucune identification" />
				</SelectTrigger>
				<SelectContent>
					{options.length === 0 ? (
						<div className="px-3 py-2 text-sm text-muted-foreground">
							Aucune identification compatible
						</div>
					) : (
						options.map((option) => (
							<SelectItem key={option.id} value={option.id}>
								{option.name}
							</SelectItem>
						))
					)}
				</SelectContent>
			</Select>
			<Button
				type="button"
				variant="outline"
				size="icon"
				title="Nouvelle identification"
				onClick={onCreateNew}
				disabled={disabled}
			>
				<Plus />
				<span className="sr-only">Nouvelle identification</span>
			</Button>
		</div>
	)
}
