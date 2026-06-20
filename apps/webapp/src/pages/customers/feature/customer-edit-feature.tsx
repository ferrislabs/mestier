import { type AnyFormApi, useForm } from '@tanstack/react-form'
import { Link } from '@tanstack/react-router'
import { AlertCircle, UserX } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	type Customer,
	type CustomerContext,
	useCreateCustomerContext,
	useCustomer,
	useCustomerContexts,
	useDeleteCustomerContext,
	useUpdateCustomer,
	useUpdateCustomerContext,
	useUploadFile,
} from '#/hooks/use-customers'
import { useDirtyBaseline } from '#/hooks/use-dirty'
import {
	type CustomerContextFormValues,
	type CustomerFormValues,
	customerContextToForm,
	customerToForm,
	EMPTY_CUSTOMER_CONTEXT_FORM,
} from '#/pages/customers/types'
import { CustomerEditUI } from '#/pages/customers/ui/customer-edit-ui'

interface CustomerEditFeatureProps {
	customerId: string
}

export function CustomerEditFeature({ customerId }: CustomerEditFeatureProps) {
	const customer = useCustomer(customerId)
	const customerContexts = useCustomerContexts(customerId)

	if (customer.isLoading) {
		return <CustomerEditUI.Loading />
	}

	if (customer.isError) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Impossible de charger le client</p>
					<p className="text-sm text-muted-foreground">
						{customer.error.message}
					</p>
				</div>
				<Button onClick={() => void customer.refetch()} variant="outline">
					Réessayer
				</Button>
			</div>
		)
	}

	if (!customer.data?.data) {
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

	return (
		<CustomerEditInner
			customer={customer.data.data}
			customerContexts={customerContexts.data?.data ?? []}
			customerContextsError={customerContexts.error?.message ?? null}
			isCustomerContextsLoading={customerContexts.isLoading}
			refetchCustomerContexts={() => void customerContexts.refetch()}
		/>
	)
}

interface CustomerEditInnerProps {
	customer: Customer
	customerContexts: CustomerContext[]
	customerContextsError: string | null
	isCustomerContextsLoading: boolean
	refetchCustomerContexts: () => void
}

function CustomerEditInner({
	customer,
	customerContexts,
	customerContextsError,
	isCustomerContextsLoading,
	refetchCustomerContexts,
}: CustomerEditInnerProps) {
	const updateCustomer = useUpdateCustomer()
	const createCustomerContext = useCreateCustomerContext()
	const updateCustomerContext = useUpdateCustomerContext()
	const deleteCustomerContext = useDeleteCustomerContext(customer.id)
	const uploadFile = useUploadFile()
	const commitRef = useRef<(v: CustomerFormValues) => void>(() => {})
	const [customerContextDraft, setCustomerContextDraft] =
		useState<CustomerContextFormValues>(EMPTY_CUSTOMER_CONTEXT_FORM)
	const [editingCustomerContextId, setEditingCustomerContextId] = useState<
		string | null
	>(null)
	const [photoPreviewUrl, setPhotoPreviewUrl] = useState<string | null>(null)

	const form = useForm({
		defaultValues: customerToForm(customer),
		onSubmit: async ({ value }) => {
			await updateCustomer.mutateAsync({
				path: { customer_id: customer.id },
				body: {
					first_name: value.firstName.trim(),
					last_name: value.lastName.trim(),
					email: value.email.trim() || null,
					phone: value.phone.trim() || null,
				},
			})
			commitRef.current(value)
		},
	})

	useEffect(() => {
		return () => {
			if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
		}
	}, [photoPreviewUrl])

	const resetCustomerContextDraft = () => {
		setEditingCustomerContextId(null)
		setCustomerContextDraft(EMPTY_CUSTOMER_CONTEXT_FORM)
		if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
		setPhotoPreviewUrl(null)
	}

	const handleCustomerContextSubmit = async () => {
		const body = {
			label: customerContextDraft.label.trim(),
			address_line: customerContextDraft.addressLine.trim() || null,
			postal_code: customerContextDraft.postalCode.trim() || null,
			city: customerContextDraft.city.trim() || null,
			photo_key: customerContextDraft.photoKey.trim() || null,
		}

		if (!body.label) return

		if (editingCustomerContextId) {
			await updateCustomerContext.mutateAsync({
				path: { customer_context_id: editingCustomerContextId },
				body,
			})
		} else {
			await createCustomerContext.mutateAsync({
				path: { customer_id: customer.id },
				body,
			})
		}

		resetCustomerContextDraft()
	}

	const handlePhotoChange = async (file: File) => {
		if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
		setPhotoPreviewUrl(URL.createObjectURL(file))
		const uploaded = await uploadFile.mutateAsync(file)
		setCustomerContextDraft((current) => ({
			...current,
			photoKey: uploaded.data.key,
		}))
	}

	return (
		<form.Subscribe
			selector={(s) => ({ values: s.values, isSubmitting: s.isSubmitting })}
		>
			{({ values, isSubmitting }) => (
				<CustomerEditForm
					customer={customer}
					values={values}
					customerContexts={customerContexts}
					customerContextsError={
						customerContextsError ??
						createCustomerContext.error?.message ??
						updateCustomerContext.error?.message ??
						deleteCustomerContext.error?.message ??
						uploadFile.error?.message ??
						null
					}
					isSubmitting={isSubmitting || updateCustomer.isPending}
					isCustomerContextsLoading={isCustomerContextsLoading}
					customerContextDraft={customerContextDraft}
					editingCustomerContextId={editingCustomerContextId}
					isCustomerContextSaving={
						createCustomerContext.isPending || updateCustomerContext.isPending
					}
					isUploadingPhoto={uploadFile.isPending}
					deletingCustomerContextId={
						deleteCustomerContext.variables?.path.customer_context_id &&
						deleteCustomerContext.isPending
							? deleteCustomerContext.variables.path.customer_context_id
							: null
					}
					photoPreviewUrl={photoPreviewUrl}
					form={form}
					commitRef={commitRef}
					onCustomerContextChange={(patch) =>
						setCustomerContextDraft((current) => ({ ...current, ...patch }))
					}
					onCustomerContextEdit={(customerContext) => {
						setEditingCustomerContextId(customerContext.id)
						setCustomerContextDraft(customerContextToForm(customerContext))
						if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
						setPhotoPreviewUrl(null)
					}}
					onCustomerContextCancel={resetCustomerContextDraft}
					onCustomerContextSubmit={() => void handleCustomerContextSubmit()}
					onCustomerContextDelete={(customerContext) =>
						deleteCustomerContext.mutate({
							path: { customer_context_id: customerContext.id },
						})
					}
					onCustomerContextPhotoChange={(file) => void handlePhotoChange(file)}
					onRetryCustomerContexts={refetchCustomerContexts}
				/>
			)}
		</form.Subscribe>
	)
}

interface CustomerEditFormProps {
	customer: Customer
	values: CustomerFormValues
	customerContexts: CustomerContext[]
	customerContextsError: string | null
	isSubmitting: boolean
	isCustomerContextsLoading: boolean
	customerContextDraft: CustomerContextFormValues
	editingCustomerContextId: string | null
	isCustomerContextSaving: boolean
	isUploadingPhoto: boolean
	deletingCustomerContextId: string | null
	photoPreviewUrl: string | null
	form: AnyFormApi
	commitRef: React.MutableRefObject<(v: CustomerFormValues) => void>
	onCustomerContextChange: (patch: Partial<CustomerContextFormValues>) => void
	onCustomerContextEdit: (customerContext: CustomerContext) => void
	onCustomerContextCancel: () => void
	onCustomerContextSubmit: () => void
	onCustomerContextDelete: (customerContext: CustomerContext) => void
	onCustomerContextPhotoChange: (file: File) => void
	onRetryCustomerContexts: () => void
}

function CustomerEditForm({
	customer,
	values,
	customerContexts,
	customerContextsError,
	isSubmitting,
	isCustomerContextsLoading,
	customerContextDraft,
	editingCustomerContextId,
	isCustomerContextSaving,
	isUploadingPhoto,
	deletingCustomerContextId,
	photoPreviewUrl,
	form,
	commitRef,
	onCustomerContextChange,
	onCustomerContextEdit,
	onCustomerContextCancel,
	onCustomerContextSubmit,
	onCustomerContextDelete,
	onCustomerContextPhotoChange,
	onRetryCustomerContexts,
}: CustomerEditFormProps) {
	const baseline = customerToForm(customer)
	const {
		isDirty,
		changedKeys,
		commit,
		reset: resetBaseline,
	} = useDirtyBaseline(baseline, values)

	commitRef.current = commit

	return (
		<CustomerEditUI
			customer={customer}
			form={values}
			isDirty={isDirty}
			changedKeys={changedKeys}
			isSaving={isSubmitting}
			customerContexts={customerContexts}
			customerContextsError={customerContextsError}
			isCustomerContextsLoading={isCustomerContextsLoading}
			customerContextDraft={customerContextDraft}
			editingCustomerContextId={editingCustomerContextId}
			isCustomerContextSaving={isCustomerContextSaving}
			isUploadingPhoto={isUploadingPhoto}
			deletingCustomerContextId={deletingCustomerContextId}
			photoPreviewUrl={photoPreviewUrl}
			onChange={(patch) => {
				for (const key of Object.keys(patch) as (keyof CustomerFormValues)[]) {
					form.setFieldValue(key, patch[key] as never)
				}
			}}
			onReset={() => {
				form.reset()
				resetBaseline()
			}}
			onSave={() => form.handleSubmit()}
			onCustomerContextChange={onCustomerContextChange}
			onCustomerContextEdit={onCustomerContextEdit}
			onCustomerContextCancel={onCustomerContextCancel}
			onCustomerContextSubmit={onCustomerContextSubmit}
			onCustomerContextDelete={onCustomerContextDelete}
			onCustomerContextPhotoChange={onCustomerContextPhotoChange}
			onRetryCustomerContexts={onRetryCustomerContexts}
		/>
	)
}
