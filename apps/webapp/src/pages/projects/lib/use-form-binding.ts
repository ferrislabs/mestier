import { useId, useRef, useState } from 'react'
import type { FormErrors } from '#/pages/projects/lib/form-model'

export type FieldBinding = {
	id: string
	error: string | undefined
	input: {
		id: string
		'aria-invalid': true | undefined
		'aria-describedby': string | undefined
		onBlur: () => void
	}
}

export function useFormBinding<K extends string>(errors: FormErrors<K>) {
	const baseId = useId()
	const [touched, setTouched] = useState<ReadonlySet<string>>(new Set())
	const [submitted, setSubmitted] = useState(false)
	const formRef = useRef<HTMLFormElement>(null)

	function bind(field: K): FieldBinding {
		const id = `${baseId}-${field}`
		const error =
			submitted || touched.has(field) ? errors.byField[field] : undefined
		return {
			id,
			error,
			input: {
				id,
				'aria-invalid': error ? true : undefined,
				'aria-describedby': error ? `${id}-error` : undefined,
				onBlur: () => setTouched((current) => new Set([...current, field])),
			},
		}
	}

	function revealAll() {
		setSubmitted(true)
		requestAnimationFrame(() =>
			formRef.current
				?.querySelector<HTMLElement>('[aria-invalid="true"]')
				?.focus(),
		)
	}

	return { baseId, bind, revealAll, formRef }
}
