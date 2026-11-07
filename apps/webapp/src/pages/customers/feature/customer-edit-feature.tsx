import { Plus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useBillingSettings } from '#/hooks/use-billing-settings'
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
import { useOutstandingByCustomer } from '#/hooks/use-invoices'
import { useQuotes } from '#/hooks/use-quotes'
import { mutationErrorMessage } from '#/lib/api-error'
import { CURRENT_PERSON_ID } from '#/pages/customers/feature/mocks/people'
import {
	useSaveStripeCustomerId,
	useStripeCustomerId,
} from '#/pages/customers/feature/mocks/stripe-customer-ids'
import { useAllInvoices } from '#/pages/customers/feature/use-all-invoices'
import { useCustomerPayments } from '#/pages/customers/feature/use-customer-payments'
import { useOrganizationPeople } from '#/pages/customers/feature/use-organization-people'
import {
	type AccountForm,
	accountFormOf,
	type ContactForm,
	profileOfForm,
	registrationNumberOf,
	stripeCustomerIdOf,
} from '#/pages/customers/lib/account-form'
import {
	accountDetailsOf,
	billingContactOf,
	billingContextOf,
	countryCodeOf,
} from '#/pages/customers/lib/accounts'
import {
	creditNoteRowsOf,
	invoiceRowsOf,
	paymentRowsOf,
	quoteRowsOf,
} from '#/pages/customers/lib/activity'
import { findPerson } from '#/pages/customers/lib/people'
import {
	activitySummary,
	addressesSummary,
	type ClientTab,
	contactsSummary,
	identitySummary,
} from '#/pages/customers/lib/rubriques'
import { customerStats } from '#/pages/customers/lib/stats'
import {
	ACCOUNT_KIND_LABELS,
	accountStatusOf,
	apiAddressKindOf,
	customerBody,
	kindOf,
} from '#/pages/customers/types'
import { ActivitySection } from '#/pages/customers/ui/activity-section'
import { AddressesSection } from '#/pages/customers/ui/addresses-section'
import { hueOf } from '#/pages/customers/ui/avatars'
import { ContactsSection } from '#/pages/customers/ui/contacts-section'
import { CustomerEditUI } from '#/pages/customers/ui/customer-edit-ui'
import { CustomerSidebar } from '#/pages/customers/ui/customer-sidebar'
import { IdentitySection } from '#/pages/customers/ui/identity-section'
import { OpportunitiesCard } from '#/pages/customers/ui/opportunities-card'
import { useDealsByCustomer } from '#/pages/deals/feature/use-deals'
import { isOpenStage } from '#/pages/deals/lib/pipeline'
import { DealCreateDialog } from '#/pages/deals/ui/deal-create-dialog'

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
	const invoices = useAllInvoices(organizationId)
	const stripeCustomerId = useStripeCustomerId(customerId)
	const settings = useBillingSettings(organizationId)
	const saveStripeCustomerId = useSaveStripeCustomerId()
	const { people } = useOrganizationPeople(organizationId)
	const updateCustomer = useUpdateCustomer()
	const createContact = useCreateCustomerContact()
	const updateContact = useUpdateCustomerContact()
	const deleteContact = useDeleteCustomerContact(customerId)
	const createContext = useCreateCustomerContext()
	const updateContext = useUpdateCustomerContext()
	const deleteContext = useDeleteCustomerContext(customerId)
	const uploadFile = useUploadFile()
	const customerDeals = useDealsByCustomer(customerId)
	const [creatingDeal, setCreatingDeal] = useState(false)

	const invoiceData = invoices.invoices
	const payments = useCustomerPayments(
		invoiceData.filter((invoice) => invoice.customer_id === customerId),
	)
	const quoteRows = quoteRowsOf(quotes.data?.data ?? [], customerId)
	const invoiceRows = invoiceRowsOf(invoiceData, customerId)
	const creditNoteRows = creditNoteRowsOf(invoiceData, customerId)
	const paymentRows = paymentRowsOf(payments.entries)

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
	const stats = customerStats({
		customerId,
		invoices: invoiceData,
		quotes: quotes.data?.data ?? [],
		outstandingCents,
		now: Date.now(),
	})
	const openDeals = customerDeals.deals.filter((deal) =>
		isOpenStage(deal.stage),
	).length
	const kind = kindOf(customer)
	const status = accountStatusOf(customer.status)
	const details = accountDetailsOf(customer, stripeCustomerId)
	const defaultTerms = settings.data?.data.payment_terms_days ?? 30

	const save = async (form: AccountForm) => {
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, {
				name: form.name.trim(),
				registration_number: registrationNumberOf(form),
				email: form.email.trim() || null,
				phone: form.phone.trim() || null,
				...profileOfForm(form),
			}),
		})
		await saveStripeCustomerId.mutateAsync(
			customer.id,
			stripeCustomerIdOf(form),
		)
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
			mobile: form.mobile.trim() || null,
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

	const convert = async () => {
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, { status: 'CLIENT', pipeline_stage: 'WON' }),
		})
	}

	const archive = async () => {
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, { status: 'ARCHIVED' }),
		})
	}

	const restore = async () => {
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, { status: 'PROSPECT' }),
		})
	}

	const designateBilling = (patch: {
		billing_contact_id?: string
		billing_address_id?: string
	}) =>
		void updateCustomer
			.mutateAsync({
				path: { customer_id: customer.id },
				body: customerBody(customer, patch),
			})
			.catch(() => undefined)

	const ignore = () => undefined

	const contactError =
		mutationErrorMessage(createContact.error) ??
		mutationErrorMessage(updateContact.error) ??
		mutationErrorMessage(deleteContact.error) ??
		mutationErrorMessage(updateCustomer.error)
	const contextError =
		mutationErrorMessage(createContext.error) ??
		mutationErrorMessage(updateContext.error) ??
		mutationErrorMessage(deleteContext.error) ??
		mutationErrorMessage(uploadFile.error) ??
		mutationErrorMessage(updateCustomer.error)

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
					details={details}
					owner={findPerson(people, details.ownerId)}
					people={people}
					defaultTerms={defaultTerms}
					initialForm={accountFormOf(customer, stripeCustomerId)}
					isSaving={updateCustomer.isPending || saveStripeCustomerId.isPending}
					saveError={mutationErrorMessage(updateCustomer.error)}
					onSave={save}
					onConvert={() => void convert().catch(ignore)}
					onArchive={() => void archive().catch(ignore)}
					onRestore={() => void restore().catch(ignore)}
					openDeals={openDeals}
				/>
			)}
			{tab === 'contacts' && (
				<ContactsSection
					contacts={contactList}
					billingContactId={
						billingContactOf(contactList, customer.billing_contact_id)?.id
					}
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
									mobile: contact.mobile ?? null,
									is_primary: true,
								},
							}),
						onMakeBilling: (contact) =>
							designateBilling({ billing_contact_id: contact.id }),
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
					billingContextId={
						billingContextOf(contextList, customer.billing_address_id)?.id
					}
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
								kind: apiAddressKindOf(draft.kind),
								address_line: draft.address.street.trim() || null,
								address_line2: draft.address.complement.trim() || null,
								postal_code: draft.address.postalCode.trim() || null,
								city: draft.address.city.trim() || null,
								country: countryCodeOf(draft.address.country),
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
						onMakeBilling: (context) =>
							designateBilling({ billing_address_id: context.id }),
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
					revenue12m={stats.revenue12m}
					outstanding={outstandingCents}
					overdue={stats.overdue}
					openDeals={openDeals}
					opportunities={
						<OpportunitiesCard
							organizationSlug={organizationSlug}
							rows={customerDeals.deals.map((deal) => ({
								id: deal.id,
								title: deal.title,
								stage: deal.stage,
								amount: deal.amount,
								date: deal.createdAt,
							}))}
							action={
								status === 'archived' ? undefined : (
									<DealCreateDialog
										open={creatingDeal}
										onOpenChange={setCreatingDeal}
										customers={[customer]}
										defaultOwnerId={CURRENT_PERSON_ID}
										organizationSlug={organizationSlug}
										onCreate={customerDeals.create}
										trigger={
											<Button variant="outline" size="sm">
												<Plus aria-hidden /> Nouvelle opportunité
											</Button>
										}
									/>
								)
							}
						/>
					}
					quotes={quoteRows}
					invoices={invoiceRows}
					creditNotes={creditNoteRows}
					payments={paymentRows}
					isLoading={quotes.isLoading || invoices.isLoading}
					isLoadingPayments={payments.isLoading}
				/>
			)}
		</CustomerEditUI>
	)
}
