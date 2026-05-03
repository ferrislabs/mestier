import { type AnyFormApi, useForm } from '@tanstack/react-form'
import { Link } from '@tanstack/react-router'
import { UserX } from 'lucide-react'
import { useRef } from 'react'
import { Button } from '#/components/ui/button'
import { useDirtyBaseline } from '#/hooks/use-dirty'
import { findCustomerById } from '#/pages/customers/mocks'
import type { Customer } from '#/pages/customers/types'
import { CustomerEditUI } from '#/pages/customers/ui/customer-edit-ui'

interface CustomerEditFeatureProps {
	customerId: string
}

export function CustomerEditFeature({ customerId }: CustomerEditFeatureProps) {
	const customer = findCustomerById(customerId)

	if (!customer) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<UserX className="size-6 text-muted-foreground" />
				</div>
				<div>
					<p className="font-semibold">Client introuvable</p>
					<p className="text-sm text-muted-foreground">
						Aucun client ne correspond à cet identifiant.
					</p>
				</div>
				<Button asChild variant="outline">
					<Link to="/customers">Retour aux clients</Link>
				</Button>
			</div>
		)
	}

	return <CustomerEditInner customer={customer} />
}

function CustomerEditInner({ customer }: { customer: Customer }) {
	const commitRef = useRef<(v: Customer) => void>(() => {})

	const form = useForm({
		defaultValues: customer,
		onSubmit: async ({ value }) => {
			await new Promise((r) => setTimeout(r, 800))
			console.log('[customers] saved', value)
			commitRef.current(value)
		},
	})

	return (
		<form.Subscribe
			selector={(s) => ({ values: s.values, isSubmitting: s.isSubmitting })}
		>
			{({ values, isSubmitting }) => (
				<CustomerEditForm
					customer={customer}
					values={values}
					isSubmitting={isSubmitting}
					form={form}
					commitRef={commitRef}
				/>
			)}
		</form.Subscribe>
	)
}

interface CustomerEditFormProps {
	customer: Customer
	values: Customer
	isSubmitting: boolean
	form: AnyFormApi
	commitRef: React.MutableRefObject<(v: Customer) => void>
}

function CustomerEditForm({
	customer,
	values,
	isSubmitting,
	form,
	commitRef,
}: CustomerEditFormProps) {
	const {
		isDirty,
		changedKeys,
		commit,
		reset: resetBaseline,
	} = useDirtyBaseline(customer, values)

	commitRef.current = commit

	return (
		<CustomerEditUI
			customer={customer}
			form={values}
			isDirty={isDirty}
			changedKeys={changedKeys}
			isSaving={isSubmitting}
			onChange={(patch) => {
				for (const key of Object.keys(patch) as (keyof Customer)[]) {
					form.setFieldValue(key, patch[key] as never)
				}
			}}
			onAddressChange={(patch) => {
				form.setFieldValue('address', { ...values.address, ...patch })
			}}
			onReset={() => {
				form.reset()
				resetBaseline()
			}}
			onSave={() => form.handleSubmit()}
		/>
	)
}
