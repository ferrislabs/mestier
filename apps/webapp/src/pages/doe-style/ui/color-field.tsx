import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { isValidHex } from '#/pages/doe-style/lib/style'

export function ColorField({
	label,
	value,
	error,
	onChange,
}: {
	label: string
	value: string
	error: string | undefined
	onChange: (value: string) => void
}) {
	const valid = isValidHex(value)
	return (
		<div className="flex flex-col gap-2">
			<Label>{label}</Label>
			<div className="flex items-center gap-2">
				<input
					type="color"
					aria-label={`${label} (sélecteur)`}
					value={valid ? value.trim().toLowerCase() : '#000000'}
					className="h-9 w-11 shrink-0 cursor-pointer rounded-md border border-input bg-transparent p-1 outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
					onChange={(event) => onChange(event.target.value)}
				/>
				<Input
					value={value}
					aria-label={label}
					aria-invalid={Boolean(error)}
					className="w-32 font-mono"
					onChange={(event) => onChange(event.target.value)}
				/>
			</div>
			{error ? (
				<p role="alert" className="text-[12px] text-state-alert">
					{error}
				</p>
			) : null}
		</div>
	)
}
