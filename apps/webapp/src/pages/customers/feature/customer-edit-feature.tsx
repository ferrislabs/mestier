import { type AnyFormApi, useForm } from '@tanstack/react-form'
import { Link } from '@tanstack/react-router'
import { AlertCircle, UserX } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	type Customer,
	type Property,
	useCreateProperty,
	useCustomer,
	useCustomerProperties,
	useDeleteProperty,
	useUpdateCustomer,
	useUpdateProperty,
	useUploadFile,
} from '#/hooks/use-customers'
import { useDirtyBaseline } from '#/hooks/use-dirty'
import {
	type CustomerFormValues,
	customerToForm,
	EMPTY_PROPERTY_FORM,
	type PropertyFormValues,
	propertyToForm,
} from '#/pages/customers/types'
import { CustomerEditUI } from '#/pages/customers/ui/customer-edit-ui'

interface CustomerEditFeatureProps {
	customerId: string
}

export function CustomerEditFeature({ customerId }: CustomerEditFeatureProps) {
	const customer = useCustomer(customerId)
	const properties = useCustomerProperties(customerId)

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
			properties={properties.data?.data ?? []}
			propertiesError={properties.error?.message ?? null}
			isPropertiesLoading={properties.isLoading}
			refetchProperties={() => void properties.refetch()}
		/>
	)
}

interface CustomerEditInnerProps {
	customer: Customer
	properties: Property[]
	propertiesError: string | null
	isPropertiesLoading: boolean
	refetchProperties: () => void
}

function CustomerEditInner({
	customer,
	properties,
	propertiesError,
	isPropertiesLoading,
	refetchProperties,
}: CustomerEditInnerProps) {
	const updateCustomer = useUpdateCustomer()
	const createProperty = useCreateProperty()
	const updateProperty = useUpdateProperty()
	const deleteProperty = useDeleteProperty(customer.id)
	const uploadFile = useUploadFile()
	const commitRef = useRef<(v: CustomerFormValues) => void>(() => {})
	const [propertyDraft, setPropertyDraft] =
		useState<PropertyFormValues>(EMPTY_PROPERTY_FORM)
	const [editingPropertyId, setEditingPropertyId] = useState<string | null>(
		null,
	)
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

	const resetPropertyDraft = () => {
		setEditingPropertyId(null)
		setPropertyDraft(EMPTY_PROPERTY_FORM)
		if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
		setPhotoPreviewUrl(null)
	}

	const handlePropertySubmit = async () => {
		const body = {
			label: propertyDraft.label.trim(),
			street: propertyDraft.street.trim(),
			zip: propertyDraft.zip.trim(),
			city: propertyDraft.city.trim(),
			photo_key: propertyDraft.photoKey.trim() || null,
		}

		if (!body.label || !body.street || !body.zip || !body.city) return

		if (editingPropertyId) {
			await updateProperty.mutateAsync({
				path: { property_id: editingPropertyId },
				body,
			})
		} else {
			await createProperty.mutateAsync({
				path: { customer_id: customer.id },
				body,
			})
		}

		resetPropertyDraft()
	}

	const handlePhotoChange = async (file: File) => {
		if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
		setPhotoPreviewUrl(URL.createObjectURL(file))
		const uploaded = await uploadFile.mutateAsync(file)
		setPropertyDraft((current) => ({
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
					properties={properties}
					propertiesError={
						propertiesError ??
						createProperty.error?.message ??
						updateProperty.error?.message ??
						deleteProperty.error?.message ??
						uploadFile.error?.message ??
						null
					}
					isSubmitting={isSubmitting || updateCustomer.isPending}
					isPropertiesLoading={isPropertiesLoading}
					propertyDraft={propertyDraft}
					editingPropertyId={editingPropertyId}
					isPropertySaving={
						createProperty.isPending || updateProperty.isPending
					}
					isUploadingPhoto={uploadFile.isPending}
					deletingPropertyId={
						deleteProperty.variables?.path.property_id &&
						deleteProperty.isPending
							? deleteProperty.variables.path.property_id
							: null
					}
					photoPreviewUrl={photoPreviewUrl}
					form={form}
					commitRef={commitRef}
					onPropertyChange={(patch) =>
						setPropertyDraft((current) => ({ ...current, ...patch }))
					}
					onPropertyEdit={(property) => {
						setEditingPropertyId(property.id)
						setPropertyDraft(propertyToForm(property))
						if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
						setPhotoPreviewUrl(null)
					}}
					onPropertyCancel={resetPropertyDraft}
					onPropertySubmit={() => void handlePropertySubmit()}
					onPropertyDelete={(property) =>
						deleteProperty.mutate({
							path: { property_id: property.id },
						})
					}
					onPropertyPhotoChange={(file) => void handlePhotoChange(file)}
					onRetryProperties={refetchProperties}
				/>
			)}
		</form.Subscribe>
	)
}

interface CustomerEditFormProps {
	customer: Customer
	values: CustomerFormValues
	properties: Property[]
	propertiesError: string | null
	isSubmitting: boolean
	isPropertiesLoading: boolean
	propertyDraft: PropertyFormValues
	editingPropertyId: string | null
	isPropertySaving: boolean
	isUploadingPhoto: boolean
	deletingPropertyId: string | null
	photoPreviewUrl: string | null
	form: AnyFormApi
	commitRef: React.MutableRefObject<(v: CustomerFormValues) => void>
	onPropertyChange: (patch: Partial<PropertyFormValues>) => void
	onPropertyEdit: (property: Property) => void
	onPropertyCancel: () => void
	onPropertySubmit: () => void
	onPropertyDelete: (property: Property) => void
	onPropertyPhotoChange: (file: File) => void
	onRetryProperties: () => void
}

function CustomerEditForm({
	customer,
	values,
	properties,
	propertiesError,
	isSubmitting,
	isPropertiesLoading,
	propertyDraft,
	editingPropertyId,
	isPropertySaving,
	isUploadingPhoto,
	deletingPropertyId,
	photoPreviewUrl,
	form,
	commitRef,
	onPropertyChange,
	onPropertyEdit,
	onPropertyCancel,
	onPropertySubmit,
	onPropertyDelete,
	onPropertyPhotoChange,
	onRetryProperties,
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
			properties={properties}
			propertiesError={propertiesError}
			isPropertiesLoading={isPropertiesLoading}
			propertyDraft={propertyDraft}
			editingPropertyId={editingPropertyId}
			isPropertySaving={isPropertySaving}
			isUploadingPhoto={isUploadingPhoto}
			deletingPropertyId={deletingPropertyId}
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
			onPropertyChange={onPropertyChange}
			onPropertyEdit={onPropertyEdit}
			onPropertyCancel={onPropertyCancel}
			onPropertySubmit={onPropertySubmit}
			onPropertyDelete={onPropertyDelete}
			onPropertyPhotoChange={onPropertyPhotoChange}
			onRetryProperties={onRetryProperties}
		/>
	)
}
