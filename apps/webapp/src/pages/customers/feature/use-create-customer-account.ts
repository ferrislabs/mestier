import { useNavigate } from '@tanstack/react-router'
import {
	useCreateCustomer,
	useCreateCustomerContact,
	useCreateCustomerContext,
	useUpdateCustomer,
} from '#/hooks/use-customers'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { useSaveStripeCustomerId } from '#/pages/customers/feature/mocks/stripe-customer-ids'
import {
	type AccountForm,
	isAddressEmpty,
	isContactEmpty,
	profileOfForm,
	registrationNumberOf,
	stripeCustomerIdOf,
} from '#/pages/customers/lib/account-form'
import { countryCodeOf } from '#/pages/customers/lib/accounts'
import { customerBody } from '#/pages/customers/types'

export function useCreateCustomerAccount(
	organizationId: string,
	organizationSlug: string,
) {
	const navigate = useNavigate()
	const createCustomer = useCreateCustomer(organizationId)
	const updateCustomer = useUpdateCustomer()
	const createContact = useCreateCustomerContact()
	const createContext = useCreateCustomerContext()
	const saveStripeCustomerId = useSaveStripeCustomerId()

	async function create(form: AccountForm) {
		const contact = isContactEmpty(form.contact) ? null : form.contact
		const response = await createCustomer.mutateAsync({
			path: { organization_id: organizationId },
			body: {
				status: form.status === 'client' ? 'CLIENT' : 'PROSPECT',
				name: form.name.trim(),
				registration_number: registrationNumberOf(form),
				email: contact?.email.trim() || null,
				phone: contact?.phone.trim() || null,
				...profileOfForm(form),
			},
		})
		const customerId = response.data.id
		let billingContactId: string | null = null
		let billingAddressId: string | null = null
		if (contact) {
			const created = await createContact.mutateAsync({
				path: { customer_id: customerId },
				body: {
					first_name: contact.firstName.trim(),
					last_name: contact.lastName.trim(),
					role: contact.role.trim() || null,
					email: contact.email.trim() || null,
					phone: contact.phone.trim() || null,
					mobile: contact.mobile.trim() || null,
					is_primary: true,
				},
			})
			billingContactId = created.data.id
		}
		if (!isAddressEmpty(form.address)) {
			const created = await createContext.mutateAsync({
				path: { customer_id: customerId },
				body: {
					label: form.kind === 'particulier' ? 'Domicile' : 'Siège',
					kind: 'BILLING',
					address_line: form.address.street.trim() || null,
					address_line2: form.address.complement.trim() || null,
					postal_code: form.address.postalCode.trim() || null,
					city: form.address.city.trim() || null,
					country: countryCodeOf(form.address.country),
					photo_key: null,
				},
			})
			billingAddressId = created.data.id
		}
		if (billingContactId || billingAddressId)
			await updateCustomer.mutateAsync({
				path: { customer_id: customerId },
				body: customerBody(response.data, {
					billing_contact_id: billingContactId,
					billing_address_id: billingAddressId,
				}),
			})
		const stripeCustomerId = stripeCustomerIdOf(form)
		if (stripeCustomerId)
			await saveStripeCustomerId.mutateAsync(customerId, stripeCustomerId)
		void navigate({
			to: buildOrgPath(organizationSlug, '/crm/clients/$customerId'),
			params: { customerId },
		})
	}

	return {
		create,
		isPending:
			createCustomer.isPending ||
			updateCustomer.isPending ||
			createContact.isPending ||
			createContext.isPending,
		error:
			mutationErrorMessage(createCustomer.error) ??
			mutationErrorMessage(createContact.error) ??
			mutationErrorMessage(createContext.error) ??
			mutationErrorMessage(updateCustomer.error) ??
			null,
	}
}
