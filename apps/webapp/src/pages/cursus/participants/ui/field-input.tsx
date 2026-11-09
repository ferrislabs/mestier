import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'

export interface FieldInputProps {
	id: string
	label: string
	value: string
	onChange: (value: string) => void
	type?: 'text' | 'email' | 'tel' | 'date' | 'number'
	disabled?: boolean
}

export function FieldInput({
	id,
	label,
	value,
	onChange,
	type = 'text',
	disabled,
}: FieldInputProps) {
	return (
		<Field label={label} htmlFor={id}>
			<Input
				id={id}
				type={type}
				value={value}
				disabled={disabled}
				onChange={(event) => onChange(event.target.value)}
			/>
		</Field>
	)
}

export function FormError({ message }: { message: string | null }) {
	if (!message) return null
	return (
		<p
			role="alert"
			className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
		>
			{message}
		</p>
	)
}
