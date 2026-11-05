import type { ComponentProps, ReactNode } from 'react'
import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'
import { TextArea } from '#/pages/customers/ui/text-area'

type FieldProps = {
	id: string
	label: string
	value: string
	onChange: (value: string) => void
	error?: string
	hint?: string
	required?: boolean
	className?: string
	placeholder?: string
}

function describedBy(id: string, error?: string): string | undefined {
	return error ? `${id}-error` : undefined
}

export function TextField({
	id,
	label,
	value,
	onChange,
	error,
	hint,
	required,
	className,
	placeholder,
	inputMode,
	suffix,
}: FieldProps & {
	inputMode?: ComponentProps<'input'>['inputMode']
	suffix?: ReactNode
}) {
	return (
		<FormField
			id={id}
			label={label}
			required={required}
			error={error}
			hint={hint}
			className={className}
		>
			<div className="relative">
				<Input
					id={id}
					value={value}
					onChange={(event) => onChange(event.target.value)}
					autoComplete="off"
					placeholder={placeholder}
					inputMode={inputMode}
					aria-invalid={error ? true : undefined}
					aria-describedby={describedBy(id, error)}
					className={cn(suffix ? 'pr-14' : undefined)}
				/>
				{suffix && (
					<span
						aria-hidden
						className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-muted-foreground text-sm"
					>
						{suffix}
					</span>
				)}
			</div>
		</FormField>
	)
}

export function TextAreaField({
	id,
	label,
	value,
	onChange,
	error,
	hint,
	required,
	className,
	placeholder,
	rows = 3,
}: FieldProps & { rows?: number }) {
	return (
		<FormField
			id={id}
			label={label}
			required={required}
			error={error}
			hint={hint}
			className={className}
		>
			<TextArea
				id={id}
				rows={rows}
				value={value}
				onChange={(event) => onChange(event.target.value)}
				placeholder={placeholder}
				aria-invalid={error ? true : undefined}
				aria-describedby={describedBy(id, error)}
			/>
		</FormField>
	)
}
