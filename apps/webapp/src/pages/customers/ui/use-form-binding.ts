import { useId, useRef } from 'react'
import type { FieldErrors, FieldKey } from '#/pages/customers/lib/account-form'
import { useTouched } from '#/pages/customers/ui/use-touched'

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

export type BindField = (field: FieldKey) => FieldBinding

export function useFormBinding(errors: FieldErrors) {
	const baseId = useId()
	const touched = useTouched()
	const formRef = useRef<HTMLFormElement>(null)
	const bind: BindField = (field) => {
		const id = `${baseId}-${field}`
		const error = touched.isShown(field) ? errors[field] : undefined
		return {
			id,
			error,
			input: {
				id,
				'aria-invalid': error ? true : undefined,
				'aria-describedby': error ? `${id}-error` : undefined,
				onBlur: () => touched.touch(field),
			},
		}
	}
	function revealAll() {
		touched.revealAll()
		requestAnimationFrame(() =>
			formRef.current
				?.querySelector<HTMLElement>('[aria-invalid="true"]')
				?.focus(),
		)
	}
	return { baseId, bind, revealAll, formRef }
}
