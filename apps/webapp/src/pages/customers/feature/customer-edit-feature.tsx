import { type AnyFormApi, useForm } from '@tanstack/react-form'
import { Link } from '@tanstack/react-router'
import { AlertCircle, UserX } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	type CustomerContact,
	type CustomerContext,
	useCreateCustomerContact,
	useCreateCustomerContext,
	useCustomer,
	useCustomerContacts,
	useCustomerContexts,
	useDeleteCustomerContact,
	useDeleteCustomerContext,
	useUpdateCustomer,
	useUpdateCustomerContact,
	useUpdateCustomerContext,
	useUploadFile,
} from '#/hooks/use-customers'
import { useDirtyBaseline } from '#/hooks/use-dirty'
import { buildOrgPath } from '#/modules/org-path'
import {
	type CustomerContactFormValues,
	type CustomerContextFormValues,
	type CustomerFormValues,
	customerContactToForm,
	customerContextToForm,
	customerToForm,
	EMPTY_CUSTOMER_CONTACT_FORM,
	EMPTY_CUSTOMER_CONTEXT_FORM,
} from '#/pages/customers/types'
import { CustomerEditUI } from '#/pages/customers/ui/customer-edit-ui'

interface CustomerEditFeatureProps {
	customerId: string
}

export function CustomerEditFeature({ customerId }: CustomerEditFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const customer = useCustomer(customerId)
	const customerContacts = useCustomerContacts(customerId)
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
					<Link to={buildOrgPath(activeOrganization.slug, '/crm/customers')}>
						Retour aux clients
					</Link>
				</Button>
			</div>
		)
	}

	return (
		<CustomerEditInner
			customer={customer.data.data}
			customerContacts={customerContacts.data?.data ?? []}
			customerContactsError={customerContacts.error?.message ?? null}
			customerContexts={customerContexts.data?.data ?? []}
			customerContextsError={customerContexts.error?.message ?? null}
			isCustomerContactsLoading={customerContacts.isLoading}
			isCustomerContextsLoading={customerContexts.isLoading}
			refetchCustomerContacts={() => void customerContacts.refetch()}
			refetchCustomerContexts={() => void customerContexts.refetch()}
		/>
	)
}

interface CustomerEditInnerProps {
	customer: Customer
	customerContacts: CustomerContact[]
	customerContactsError: string | null
	customerContexts: CustomerContext[]
	customerContextsError: string | null
	isCustomerContactsLoading: boolean
	isCustomerContextsLoading: boolean
	refetchCustomerContacts: () => void
	refetchCustomerContexts: () => void
}

function CustomerEditInner({
	customer,
	customerContacts,
	customerContactsError,
	customerContexts,
	customerContextsError,
	isCustomerContactsLoading,
	isCustomerContextsLoading,
	refetchCustomerContacts,
	refetchCustomerContexts,
}: CustomerEditInnerProps) {
	const updateCustomer = useUpdateCustomer()
	const createCustomerContact = useCreateCustomerContact()
	const updateCustomerContact = useUpdateCustomerContact()
	const deleteCustomerContact = useDeleteCustomerContact(customer.id)
	const createCustomerContext = useCreateCustomerContext()
	const updateCustomerContext = useUpdateCustomerContext()
	const deleteCustomerContext = useDeleteCustomerContext(customer.id)
	const uploadFile = useUploadFile()
	const commitRef = useRef<(v: CustomerFormValues) => void>(() => {})
	const [customerContactDraft, setCustomerContactDraft] =
		useState<CustomerContactFormValues>(EMPTY_CUSTOMER_CONTACT_FORM)
	const [editingCustomerContactId, setEditingCustomerContactId] = useState<
		string | null
	>(null)
	const [contactDialogOpen, setContactDialogOpen] = useState(false)
	const [customerContextDraft, setCustomerContextDraft] =
		useState<CustomerContextFormValues>(EMPTY_CUSTOMER_CONTEXT_FORM)
	const [editingCustomerContextId, setEditingCustomerContextId] = useState<
		string | null
	>(null)
	const [contextDialogOpen, setContextDialogOpen] = useState(false)
	const [photoPreviewUrl, setPhotoPreviewUrl] = useState<string | null>(null)

	const form = useForm({
		defaultValues: customerToForm(customer),
		onSubmit: async ({ value }) => {
			await updateCustomer.mutateAsync({
				path: { customer_id: customer.id },
				body: {
					status: value.status,
					pipeline_stage: value.pipelineStage,
					name: value.name.trim(),
					registration_number: value.registrationNumber.trim() || null,
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

	const resetCustomerContactDraft = () => {
		setEditingCustomerContactId(null)
		setCustomerContactDraft(EMPTY_CUSTOMER_CONTACT_FORM)
	}

	const resetCustomerContextDraft = () => {
		setEditingCustomerContextId(null)
		setCustomerContextDraft(EMPTY_CUSTOMER_CONTEXT_FORM)
		if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
		setPhotoPreviewUrl(null)
	}

	const handleContactDialogOpenChange = (open: boolean) => {
		setContactDialogOpen(open)
		if (!open) resetCustomerContactDraft()
	}

	const handleContextDialogOpenChange = (open: boolean) => {
		setContextDialogOpen(open)
		if (!open) resetCustomerContextDraft()
	}

	const handleCustomerContactSubmit = async () => {
		const body = {
			first_name: customerContactDraft.firstName.trim(),
			last_name: customerContactDraft.lastName.trim(),
			role: customerContactDraft.role.trim() || null,
			email: customerContactDraft.email.trim() || null,
			phone: customerContactDraft.phone.trim() || null,
			is_primary: customerContactDraft.isPrimary,
		}

		if (!body.first_name || !body.last_name) return

		if (editingCustomerContactId) {
			await updateCustomerContact.mutateAsync({
				path: { customer_contact_id: editingCustomerContactId },
				body,
			})
		} else {
			await createCustomerContact.mutateAsync({
				path: { customer_id: customer.id },
				body,
			})
		}

		resetCustomerContactDraft()
		setContactDialogOpen(false)
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
		setContextDialogOpen(false)
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
					customerContacts={customerContacts}
					customerContactsError={
						customerContactsError ??
						createCustomerContact.error?.message ??
						updateCustomerContact.error?.message ??
						deleteCustomerContact.error?.message ??
						null
					}
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
					isCustomerContactsLoading={isCustomerContactsLoading}
					isCustomerContextsLoading={isCustomerContextsLoading}
					customerContactDraft={customerContactDraft}
					editingCustomerContactId={editingCustomerContactId}
					contactDialogOpen={contactDialogOpen}
					isCustomerContactSaving={
						createCustomerContact.isPending || updateCustomerContact.isPending
					}
					customerContextDraft={customerContextDraft}
					editingCustomerContextId={editingCustomerContextId}
					contextDialogOpen={contextDialogOpen}
					isCustomerContextSaving={
						createCustomerContext.isPending || updateCustomerContext.isPending
					}
					isUploadingPhoto={uploadFile.isPending}
					deletingCustomerContactId={
						deleteCustomerContact.variables?.path.customer_contact_id &&
						deleteCustomerContact.isPending
							? deleteCustomerContact.variables.path.customer_contact_id
							: null
					}
					deletingCustomerContextId={
						deleteCustomerContext.variables?.path.customer_context_id &&
						deleteCustomerContext.isPending
							? deleteCustomerContext.variables.path.customer_context_id
							: null
					}
					photoPreviewUrl={photoPreviewUrl}
					onContactDialogOpenChange={handleContactDialogOpenChange}
					onContextDialogOpenChange={handleContextDialogOpenChange}
					form={form}
					commitRef={commitRef}
					onPromoteToClient={() => {
						const promotedValues = {
							...values,
							status: 'CLIENT' as CustomerFormValues['status'],
							pipelineStage: 'WON' as CustomerFormValues['pipelineStage'],
						}

						updateCustomer.mutate(
							{
								path: { customer_id: customer.id },
								body: {
									status: promotedValues.status,
									pipeline_stage: promotedValues.pipelineStage,
									name: promotedValues.name.trim(),
									registration_number:
										promotedValues.registrationNumber.trim() || null,
									email: promotedValues.email.trim() || null,
									phone: promotedValues.phone.trim() || null,
								},
							},
							{
								onSuccess: () => {
									form.setFieldValue('status', promotedValues.status)
									form.setFieldValue(
										'pipelineStage',
										promotedValues.pipelineStage,
									)
									commitRef.current(promotedValues)
								},
							},
						)
					}}
					onCustomerContactChange={(patch) =>
						setCustomerContactDraft((current) => ({ ...current, ...patch }))
					}
					onCustomerContactEdit={(customerContact) => {
						setEditingCustomerContactId(customerContact.id)
						setCustomerContactDraft(customerContactToForm(customerContact))
						setContactDialogOpen(true)
					}}
					onCustomerContactCancel={() => handleContactDialogOpenChange(false)}
					onCustomerContactSubmit={() => void handleCustomerContactSubmit()}
					onCustomerContactDelete={(customerContact) =>
						deleteCustomerContact.mutate({
							path: { customer_contact_id: customerContact.id },
						})
					}
					onCustomerContextChange={(patch) =>
						setCustomerContextDraft((current) => ({ ...current, ...patch }))
					}
					onCustomerContextEdit={(customerContext) => {
						setEditingCustomerContextId(customerContext.id)
						setCustomerContextDraft(customerContextToForm(customerContext))
						if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl)
						setPhotoPreviewUrl(null)
						setContextDialogOpen(true)
					}}
					onCustomerContextCancel={() => handleContextDialogOpenChange(false)}
					onCustomerContextSubmit={() => void handleCustomerContextSubmit()}
					onCustomerContextDelete={(customerContext) =>
						deleteCustomerContext.mutate({
							path: { customer_context_id: customerContext.id },
						})
					}
					onCustomerContextPhotoChange={(file) => void handlePhotoChange(file)}
					onRetryCustomerContacts={refetchCustomerContacts}
					onRetryCustomerContexts={refetchCustomerContexts}
				/>
			)}
		</form.Subscribe>
	)
}

interface CustomerEditFormProps {
	customer: Customer
	values: CustomerFormValues
	customerContacts: CustomerContact[]
	customerContactsError: string | null
	customerContexts: CustomerContext[]
	customerContextsError: string | null
	isSubmitting: boolean
	isCustomerContactsLoading: boolean
	isCustomerContextsLoading: boolean
	customerContactDraft: CustomerContactFormValues
	editingCustomerContactId: string | null
	contactDialogOpen: boolean
	isCustomerContactSaving: boolean
	customerContextDraft: CustomerContextFormValues
	editingCustomerContextId: string | null
	contextDialogOpen: boolean
	isCustomerContextSaving: boolean
	isUploadingPhoto: boolean
	deletingCustomerContactId: string | null
	deletingCustomerContextId: string | null
	photoPreviewUrl: string | null
	onContactDialogOpenChange: (open: boolean) => void
	onContextDialogOpenChange: (open: boolean) => void
	form: AnyFormApi
	commitRef: React.MutableRefObject<(v: CustomerFormValues) => void>
	onCustomerContactChange: (patch: Partial<CustomerContactFormValues>) => void
	onCustomerContactEdit: (customerContact: CustomerContact) => void
	onCustomerContactCancel: () => void
	onCustomerContactSubmit: () => void
	onCustomerContactDelete: (customerContact: CustomerContact) => void
	onPromoteToClient: () => void
	onCustomerContextChange: (patch: Partial<CustomerContextFormValues>) => void
	onCustomerContextEdit: (customerContext: CustomerContext) => void
	onCustomerContextCancel: () => void
	onCustomerContextSubmit: () => void
	onCustomerContextDelete: (customerContext: CustomerContext) => void
	onCustomerContextPhotoChange: (file: File) => void
	onRetryCustomerContacts: () => void
	onRetryCustomerContexts: () => void
}

function CustomerEditForm({
	customer,
	values,
	customerContacts,
	customerContactsError,
	customerContexts,
	customerContextsError,
	isSubmitting,
	isCustomerContactsLoading,
	isCustomerContextsLoading,
	customerContactDraft,
	editingCustomerContactId,
	contactDialogOpen,
	isCustomerContactSaving,
	customerContextDraft,
	editingCustomerContextId,
	contextDialogOpen,
	isCustomerContextSaving,
	isUploadingPhoto,
	deletingCustomerContactId,
	deletingCustomerContextId,
	photoPreviewUrl,
	onContactDialogOpenChange,
	onContextDialogOpenChange,
	form,
	commitRef,
	onCustomerContactChange,
	onCustomerContactEdit,
	onCustomerContactCancel,
	onCustomerContactSubmit,
	onCustomerContactDelete,
	onPromoteToClient,
	onCustomerContextChange,
	onCustomerContextEdit,
	onCustomerContextCancel,
	onCustomerContextSubmit,
	onCustomerContextDelete,
	onCustomerContextPhotoChange,
	onRetryCustomerContacts,
	onRetryCustomerContexts,
}: CustomerEditFormProps) {
	const { activeOrganization } = useActiveOrganization()
	const organizationSlug = activeOrganization.slug
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
			organizationSlug={organizationSlug}
			customer={customer}
			form={values}
			isDirty={isDirty}
			changedKeys={changedKeys}
			isSaving={isSubmitting}
			customerContacts={customerContacts}
			customerContactsError={customerContactsError}
			isCustomerContactsLoading={isCustomerContactsLoading}
			customerContactDraft={customerContactDraft}
			editingCustomerContactId={editingCustomerContactId}
			contactDialogOpen={contactDialogOpen}
			isCustomerContactSaving={isCustomerContactSaving}
			deletingCustomerContactId={deletingCustomerContactId}
			customerContexts={customerContexts}
			customerContextsError={customerContextsError}
			isCustomerContextsLoading={isCustomerContextsLoading}
			customerContextDraft={customerContextDraft}
			editingCustomerContextId={editingCustomerContextId}
			contextDialogOpen={contextDialogOpen}
			isCustomerContextSaving={isCustomerContextSaving}
			isUploadingPhoto={isUploadingPhoto}
			deletingCustomerContextId={deletingCustomerContextId}
			photoPreviewUrl={photoPreviewUrl}
			onContactDialogOpenChange={onContactDialogOpenChange}
			onContextDialogOpenChange={onContextDialogOpenChange}
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
			onPromoteToClient={onPromoteToClient}
			onCustomerContactChange={onCustomerContactChange}
			onCustomerContactEdit={onCustomerContactEdit}
			onCustomerContactCancel={onCustomerContactCancel}
			onCustomerContactSubmit={onCustomerContactSubmit}
			onCustomerContactDelete={onCustomerContactDelete}
			onCustomerContextChange={onCustomerContextChange}
			onCustomerContextEdit={onCustomerContextEdit}
			onCustomerContextCancel={onCustomerContextCancel}
			onCustomerContextSubmit={onCustomerContextSubmit}
			onCustomerContextDelete={onCustomerContextDelete}
			onCustomerContextPhotoChange={onCustomerContextPhotoChange}
			onRetryCustomerContacts={onRetryCustomerContacts}
			onRetryCustomerContexts={onRetryCustomerContexts}
		/>
	)
}
