import { Input } from '#/components/ui/input'
import { Textarea } from '#/components/ui/textarea'
import { cn } from '#/lib/utils'
import type { QuestionKind } from '#/pages/survey/types'
import { DateField } from '#/pages/survey-respond/ui/date-field'

type FieldKind = Exclude<
	QuestionKind,
	'SINGLE_CHOICE' | 'MULTIPLE_CHOICE' | 'PHOTO'
>

const FIELD_PROPS: Record<
	Exclude<FieldKind, 'LONG_TEXT' | 'DATE' | 'DATETIME'>,
	{
		type: string
		inputMode?: 'text' | 'decimal' | 'tel' | 'email'
		autoComplete?: string
		placeholder?: string
	}
> = {
	TEXT: { type: 'text', inputMode: 'text' },
	NUMBER: { type: 'text', inputMode: 'decimal', placeholder: 'Par exemple 12' },
	PHONE: {
		type: 'tel',
		inputMode: 'tel',
		autoComplete: 'tel',
		placeholder: '06 12 34 56 78',
	},
	EMAIL: {
		type: 'email',
		inputMode: 'email',
		autoComplete: 'email',
		placeholder: 'nom@exemple.fr',
	},
}

const FIELD_CLASS =
	'h-12 rounded-lg bg-card px-4 text-base shadow-none md:text-base'

export function AnswerInput({
	id,
	kind,
	value,
	invalid,
	describedBy,
	required,
	onChange,
	onSubmit,
}: {
	id: string
	kind: FieldKind
	value: string
	invalid: boolean
	describedBy: string | undefined
	required: boolean
	onChange: (value: string) => void
	onSubmit: () => void
}) {
	if (kind === 'LONG_TEXT') {
		return (
			<Textarea
				id={id}
				value={value}
				rows={5}
				required={required}
				aria-invalid={invalid || undefined}
				aria-describedby={describedBy}
				className="min-h-36 rounded-lg px-4 py-3 text-base shadow-none md:text-base"
				onChange={(event) => onChange(event.target.value)}
			/>
		)
	}
	if (kind === 'DATE' || kind === 'DATETIME') {
		return (
			<DateField
				id={id}
				withTime={kind === 'DATETIME'}
				value={value}
				invalid={invalid}
				describedBy={describedBy}
				onChange={onChange}
			/>
		)
	}
	const props = FIELD_PROPS[kind]
	return (
		<Input
			id={id}
			value={value}
			type={props.type}
			inputMode={props.inputMode}
			autoComplete={props.autoComplete ?? 'off'}
			placeholder={props.placeholder}
			required={required}
			enterKeyHint="next"
			aria-invalid={invalid || undefined}
			aria-describedby={describedBy}
			className={cn(FIELD_CLASS, kind === 'NUMBER' && 'tabular-nums')}
			onChange={(event) => onChange(event.target.value)}
			onKeyDown={(event) => {
				if (event.key === 'Enter') {
					event.preventDefault()
					onSubmit()
				}
			}}
		/>
	)
}

export function isFieldKind(kind: QuestionKind): kind is FieldKind {
	return (
		kind !== 'SINGLE_CHOICE' && kind !== 'MULTIPLE_CHOICE' && kind !== 'PHOTO'
	)
}
