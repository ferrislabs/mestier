import { Plus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
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
import { useOutstandingByCustomer } from '#/hooks/use-invoices'
import { useQuotes } from '#/hooks/use-quotes'
import { mutationErrorMessage } from '#/lib/api-error'
import { useCrmSettings } from '#/pages/crm-settings/feature/mocks/settings'
import {
	useAccountExtras,
	useSaveAccountExtras,
} from '#/pages/customers/feature/mocks/account-extras'
import { CURRENT_PERSON_ID } from '#/pages/customers/feature/mocks/people'
import { useAllInvoices } from '#/pages/customers/feature/use-all-invoices'
import { addressLineOf } from '#/pages/customers/feature/use-create-customer-account'
import { useCustomerPayments } from '#/pages/customers/feature/use-customer-payments'
import {
	type AccountForm,
	accountFormOf,
	type ContactForm,
	registrationNumberOf,
} from '#/pages/customers/lib/account-form'
import {
	billingContactOf,
	billingContextOf,
	withEntry,
} from '#/pages/customers/lib/accounts'
import {
	creditNoteRowsOf,
	invoiceRowsOf,
	paymentRowsOf,
	quoteRowsOf,
} from '#/pages/customers/lib/activity'
import { todayIso } from '#/pages/customers/lib/format'
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
	const kind = kindOf(customer, extras)
	const status = accountStatusOf(customer.status)
	const defaultTerms = settings.data.settings.paymentTermsDays

	const save = async (form: AccountForm) => {
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, {
				name: form.name.trim(),
				registration_number: registrationNumberOf(form),
				email: form.email.trim() || null,
				phone: form.phone.trim() || null,
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
		const contactId = contact
			? (
					await updateContact.mutateAsync({
						path: { customer_contact_id: contact.id },
						body,
					})
				).data.id
			: (
					await createContact.mutateAsync({
						path: { customer_id: customer.id },
						body,
					})
				).data.id
		await saveExtras.mutateAsync(customer.id, {
			mobiles: withEntry(extras.mobiles, contactId, form.mobile.trim()),
		})
	}

	const convert = async () => {
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, { status: 'CLIENT', pipeline_stage: 'WON' }),
		})
		await saveExtras.mutateAsync(customer.id, { convertedAt: todayIso() })
	}

	const archive = async () => {
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, { status: 'ARCHIVED' }),
		})
		await saveExtras.mutateAsync(customer.id, { archivedAt: todayIso() })
	}

	const restore = async () => {
		await updateCustomer.mutateAsync({
			path: { customer_id: customer.id },
			body: customerBody(customer, reactivatedStatus(customer)),
		})
		await saveExtras.mutateAsync(customer.id, { archivedAt: null })
	}

	const ignore = () => undefined

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
					onConvert={() => void convert().catch(ignore)}
					onArchive={() => void archive().catch(ignore)}
					onRestore={() => void restore().catch(ignore)}
					openDeals={openDeals}
				/>
			)}
			{tab === 'contacts' && (
				<ContactsSection
					contacts={contactList}
					mobiles={extras.mobiles}
					billingContactId={billingContactOf(contactList, extras)?.id}
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
						onMakeBilling: (contact) =>
							void saveExtras.mutateAsync(customer.id, {
								billingContactId: contact.id,
							}),
						onDelete: (contact) =>
							deleteContact.mutate(
								{ path: { customer_contact_id: contact.id } },
								{
									onSuccess: () =>
										void saveExtras.mutateAsync(customer.id, {
											mobiles: withEntry(extras.mobiles, contact.id, ''),
											billingContactId:
												extras.billingContactId === contact.id
													? null
													: extras.billingContactId,
										}),
								},
							),
					}}
				/>
			)}
			{tab === 'adresses' && (
				<AddressesSection
					contexts={contextList}
					extras={extras}
					billingContextId={billingContextOf(contextList, extras)?.id}
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
							const contextId = context
								? (
										await updateContext.mutateAsync({
											path: { customer_context_id: context.id },
											body,
										})
									).data.id
								: (
										await createContext.mutateAsync({
											path: { customer_id: customer.id },
											body,
										})
									).data.id
							await saveExtras.mutateAsync(customer.id, {
								addressKinds: {
									...extras.addressKinds,
									[contextId]: draft.kind,
								},
								addressCountries: withEntry(
									extras.addressCountries,
									contextId,
									draft.address.country.trim(),
								),
							})
						},
						onMakeBilling: (context) =>
							void saveExtras.mutateAsync(customer.id, {
								billingContextId: context.id,
							}),
						onDelete: (context) =>
							deleteContext.mutate(
								{ path: { customer_context_id: context.id } },
								{
									onSuccess: () =>
										void saveExtras.mutateAsync(customer.id, {
											addressKinds: withEntry(
												extras.addressKinds,
												context.id,
												'',
											),
											addressCountries: withEntry(
												extras.addressCountries,
												context.id,
												'',
											),
											billingContextId:
												extras.billingContextId === context.id
													? null
													: extras.billingContextId,
										}),
								},
							),
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
