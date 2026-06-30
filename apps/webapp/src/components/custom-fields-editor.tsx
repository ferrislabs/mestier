import { Plus, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'

export type CustomField = { key: string; value: string }

interface CustomFieldsEditorProps {
	fields: CustomField[]
	onChange: (fields: CustomField[]) => void
	disabled?: boolean
}

export function customFieldsToRecord(
	fields: CustomField[],
): Record<string, string> {
	const record: Record<string, string> = {}
	for (const field of fields) {
		const trimmed = field.key.trim()
		if (trimmed) {
			record[trimmed] = field.value
		}
	}
	return record
}

export function recordToCustomFields(
	record: Record<string, string>,
): CustomField[] {
	return Object.entries(record).map(([key, value]) => ({ key, value }))
}

export function CustomFieldsEditor({
	fields,
	onChange,
	disabled,
}: CustomFieldsEditorProps) {
	const handleKeyChange = (index: number, key: string) => {
		const next = fields.map((field, i) =>
			i === index ? { ...field, key } : field,
		)
		onChange(next)
	}

	const handleValueChange = (index: number, value: string) => {
		const next = fields.map((field, i) =>
			i === index ? { ...field, value } : field,
		)
		onChange(next)
	}

	const handleRemove = (index: number) => {
		onChange(fields.filter((_, i) => i !== index))
	}

	const handleAdd = () => {
		onChange([...fields, { key: '', value: '' }])
	}

	return (
		<div className="flex flex-col gap-2">
			<Label>Champs personnalisés</Label>
			{fields.length > 0 ? (
				<div className="flex flex-col gap-2">
					{fields.map((field, index) => (
						// biome-ignore lint/suspicious/noArrayIndexKey: stable ordered list, index is fine here
						<div key={index} className="flex items-center gap-2">
							<Input
								value={field.key}
								onChange={(event) => handleKeyChange(index, event.target.value)}
								placeholder="Clé"
								disabled={disabled}
								className="flex-1"
							/>
							<Input
								value={field.value}
								onChange={(event) =>
									handleValueChange(index, event.target.value)
								}
								placeholder="Valeur"
								disabled={disabled}
								className="flex-1"
							/>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								onClick={() => handleRemove(index)}
								disabled={disabled}
							>
								<Trash2 className="size-4" />
								<span className="sr-only">Supprimer le champ</span>
							</Button>
						</div>
					))}
				</div>
			) : null}
			<Button
				type="button"
				variant="outline"
				size="sm"
				onClick={handleAdd}
				disabled={disabled}
				className="self-start"
			>
				<Plus className="size-4" />
				Ajouter un champ
			</Button>
		</div>
	)
}
