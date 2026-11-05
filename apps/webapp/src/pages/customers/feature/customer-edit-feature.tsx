import { useMemo } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
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
import { useInvoices, useOutstandingByCustomer } from '#/hooks/use-invoices'
import { useQuotes } from '#/hooks/use-quotes'
import { mutationErrorMessage } from '#/lib/api-error'
import { useCrmSettings } from '#/pages/crm-settings/feature/mocks/settings'
import {
	useAccountExtras,
	useSaveAccountExtras,
} from '#/pages/customers/feature/mocks/account-extras'
import { addressLineOf } from '#/pages/customers/feature/use-create-customer-account'
import {
	type AccountForm,
	accountFormOf,
	type ContactForm,
} from '#/pages/customers/lib/account-form'
import { invoiceRowsOf, quoteRowsOf } from '#/pages/customers/lib/activity'
import {
	activitySummary,
	addressesSummary,
	type ClientTab,
	contactsSummary,
	identitySummary,
} from '#/pages/customers/lib/rubriques'
import {
	ACCOUNT_KIND_LABELS,
	accountStatusOf,
	customerBody,
	kindOf,
	reactivatedStatus,
} from '#/pages/customers/types'
import { ActivitySection } from '#/pages/customers/ui/activity-section'
import { AddressesSection } from '#/pages/customers/ui/addresses-section'
import { hueOf } from '#/pages/customers/ui/avatars'
import { ContactsSection } from '#/pages/customers/ui/contacts-section'
import { CustomerEditUI } from '#/pages/customers/ui/customer-edit-ui'
import { CustomerSidebar } from '#/pages/customers/ui/customer-sidebar'
import { IdentitySection } from '#/pages/customers/ui/identity-section'

interface CustomerEditFeatureProps {
	customerId: string
	tab: ClientTab
}

export function CustomerEditFeature({
	customerId,
	tab,
}: CustomerEditFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const organizationSlug = activeOrganization.slug
	const customerQuery = useCustomer(customerId)
	const contacts = useCustomerContacts(customerId)
	const contexts = useCustomerContexts(customerId)
	const outstanding = useOutstandingByCustomer(organizationId)
	const quotes = useQuotes(organizationId)
	const invoices = useInvoices(organizationId)
	const extras = useAccountExtras(customerId)
	const settings = useCrmSettings()
	const saveExtras = useSaveAccountExtras()
	const updateCustomer = useUpdateCustomer()
	const createContact = useCreateCustomerContact()
	const updateContact = useUpdateCustomerContact()
	const deleteContact = useDeleteCustomerContact(customerId)
	const createContext = useCreateCustomerContext()
	const updateContext = useUpdateCustomerContext()
	const deleteContext = useDeleteCustomerContext(customerId)
	const uploadFile = useUploadFile()

	const quoteData = quotes.data?.data
	const invoiceData = invoices.data?.data
	const quoteRows = useMemo(
		() => quoteRowsOf(quoteData ?? [], customerId),
		[quoteData, customerId],
	)
	const invoiceRows = useMemo(
		() => invoiceRowsOf(invoiceData ?? [], customerId),
		[invoiceData, customerId],
	)

	if (customerQuery.isLoading)
		return <CustomerEditUI.Loading organizationSlug={organizationSlug} />
	if (customerQuery.isError)
		return (
			<CustomerEditUI.LoadError
				organizationSlug={organizationSlug}
				message={customerQuery.error.message}
				onRetry={() => void customerQuery.refetch()}
			/>
		)
	const customer = customerQuery.data?.data
	if (!customer)
		return <CustomerEditUI.NotFound organizationSlug={organizationSlug} />

	const contactList = contacts.data?.data ?? []
	const contextList = contexts.data?.data ?? []
	const balance = outstanding.data?.data.find(
		(row) => row.customer_id === customer.id,
	)
	const outstandingCents = balance?.outstanding_cents ?? 0
	const oldestDueAt = balance?.oldest_due_at ?? null
	const kind = kindOf(customer, extras)
	const status = accountStatusOf(customer.status)
	const defaultTerms = settings.data.settings.paymentTermsDays

	const save = async (form: AccountForm) => {
		const stage = form.pipelineStage
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, {
				name: form.name.trim(),
				registration_number:
					form.kind === 'professionnel'
						? form.registrationNumber.trim() || null
						: null,
				email: form.email.trim() || null,
				phone: form.phone.trim() || null,
				pipeline_stage: stage,
				...(stage === 'WON' ? { status: 'CLIENT' as const } : {}),
			}),
		})
		const stripe = form.stripeCustomerId.trim()
		const terms = form.paymentTerms.trim()
		await saveExtras.mutateAsync(customer.id, {
			kind: form.kind,
			vatNumber: form.vatNumber.trim(),
			website: form.website.trim(),
			ownerId: form.ownerId,
			source: form.source,
			paymentTermsDays: terms === '' ? null : Number(terms),
			stripeCustomerId: stripe === '' ? null : stripe,
			notes: form.notes.trim(),
		})
	}

	const saveContact = async (
		contact: { id: string; is_primary: boolean } | null,
		form: ContactForm,
	) => {
		const body = {
			first_name: form.firstName.trim(),
			last_name: form.lastName.trim(),
			role: form.role.trim() || null,
			email: form.email.trim() || null,
			phone: form.phone.trim() || null,
			is_primary: contact?.is_primary ?? contactList.length === 0,
		}
		if (contact)
			await updateContact.mutateAsync({
				path: { customer_contact_id: contact.id },
				body,
			})
		else
			await createContact.mutateAsync({
				path: { customer_id: customer.id },
				body,
			})
	}

	const contactError =
		mutationErrorMessage(createContact.error) ??
		mutationErrorMessage(updateContact.error) ??
		mutationErrorMessage(deleteContact.error)
	const contextError =
		mutationErrorMessage(createContext.error) ??
		mutationErrorMessage(updateContext.error) ??
		mutationErrorMessage(deleteContext.error) ??
		mutationErrorMessage(uploadFile.error)

	const summaries = {
		identite: identitySummary(customer, kind, contextList),
		contacts: contactsSummary(contactList.length),
		adresses: addressesSummary(contextList.length),
		activite: activitySummary(outstandingCents),
	}

	return (
		<CustomerEditUI
			sidebar={
				<CustomerSidebar
					organizationSlug={organizationSlug}
					customerId={customer.id}
					name={customer.name}
					hue={hueOf(customer.id)}
					status={status}
					kindLabel={ACCOUNT_KIND_LABELS[kind]}
					tab={tab}
					summaries={summaries}
				/>
			}
		>
			{tab === 'identite' && (
				<IdentitySection
					customer={customer}
					extras={extras}
					defaultTerms={defaultTerms}
					initialForm={accountFormOf(customer, extras, kind)}
					isSaving={updateCustomer.isPending || saveExtras.isPending}
					saveError={mutationErrorMessage(updateCustomer.error)}
					onSave={save}
					onConvert={() =>
						updateCustomer.mutate({
							path: { customer_id: customer.id },
							body: customerBody(customer, {
								status: 'CLIENT',
								pipeline_stage: 'WON',
							}),
						})
					}
					onArchive={() =>
						updateCustomer.mutate({
							path: { customer_id: customer.id },
							body: customerBody(customer, { status: 'ARCHIVED' }),
						})
					}
					onRestore={() =>
						updateCustomer.mutate({
							path: { customer_id: customer.id },
							body: customerBody(customer, reactivatedStatus(customer)),
						})
					}
				/>
			)}
			{tab === 'contacts' && (
				<ContactsSection
					contacts={contactList}
					isLoading={contacts.isLoading}
					loadError={mutationErrorMessage(contacts.error)}
					onRetry={() => void contacts.refetch()}
					actions={{
						isSaving: createContact.isPending || updateContact.isPending,
						isDeleting: (contactId) =>
							deleteContact.isPending &&
							deleteContact.variables?.path.customer_contact_id === contactId,
						error: contactError,
						onSave: (contact, form) => saveContact(contact, form),
						onMakePrimary: (contact) =>
							updateContact.mutate({
								path: { customer_contact_id: contact.id },
								body: {
									first_name: contact.first_name,
									last_name: contact.last_name,
									role: contact.role ?? null,
									email: contact.email ?? null,
									phone: contact.phone ?? null,
									is_primary: true,
								},
							}),
						onDelete: (contact) =>
							deleteContact.mutate({
								path: { customer_contact_id: contact.id },
							}),
					}}
				/>
			)}
			{tab === 'adresses' && (
				<AddressesSection
					contexts={contextList}
					isLoading={contexts.isLoading}
					loadError={mutationErrorMessage(contexts.error)}
					onRetry={() => void contexts.refetch()}
					actions={{
						isSaving: createContext.isPending || updateContext.isPending,
						isUploadingPhoto: uploadFile.isPending,
						isDeleting: (contextId) =>
							deleteContext.isPending &&
							deleteContext.variables?.path.customer_context_id === contextId,
						error: contextError,
						onUploadPhoto: async (file) => {
							const uploaded = await uploadFile.mutateAsync(file)
							return uploaded.data.key
						},
						onSave: async (context, draft) => {
							const body = {
								label: draft.label.trim(),
								address_line:
									addressLineOf(
										draft.address.street,
										draft.address.complement,
									) || null,
								postal_code: draft.address.postalCode.trim() || null,
								city: draft.address.city.trim() || null,
								photo_key: draft.photoKey.trim() || null,
							}
							if (context)
								await updateContext.mutateAsync({
									path: { customer_context_id: context.id },
									body,
								})
							else
								await createContext.mutateAsync({
									path: { customer_id: customer.id },
									body,
								})
						},
						onDelete: (context) =>
							deleteContext.mutate({
								path: { customer_context_id: context.id },
							}),
					}}
				/>
			)}
			{tab === 'activite' && (
				<ActivitySection
					organizationSlug={organizationSlug}
					outstanding={outstandingCents}
					oldestDueAt={oldestDueAt}
					late={
						oldestDueAt !== null && new Date(oldestDueAt).getTime() < Date.now()
					}
					quotes={quoteRows}
					invoices={invoiceRows}
					isLoading={quotes.isLoading || invoices.isLoading}
				/>
			)}
		</CustomerEditUI>
	)
}
