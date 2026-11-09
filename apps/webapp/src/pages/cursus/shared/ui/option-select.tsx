import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'

const NONE = '__none__'

interface OptionSelectProps {
	id: string
	value: string | null
	options: SelectOption[]
	placeholder: string
	noneLabel?: string
	disabled?: boolean
	className?: string
	onChange: (value: string | null) => void
}

export function OptionSelect({
	id,
	value,
	options,
	placeholder,
	noneLabel,
	disabled,
	className,
	onChange,
}: OptionSelectProps) {
	return (
		<Select
			value={value ?? (noneLabel ? NONE : '')}
			disabled={disabled}
			onValueChange={(next) => onChange(next === NONE ? null : next)}
		>
			<SelectTrigger id={id} className={className}>
				<SelectValue placeholder={placeholder} />
			</SelectTrigger>
			<SelectContent>
				{noneLabel ? <SelectItem value={NONE}>{noneLabel}</SelectItem> : null}
				{options.map((option) => (
					<SelectItem key={option.value} value={option.value}>
						{option.label}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	)
}
