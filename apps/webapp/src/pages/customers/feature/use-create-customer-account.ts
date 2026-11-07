import { useNavigate } from '@tanstack/react-router'
import {
	useCreateCustomer,
	useCreateCustomerContact,
	useCreateCustomerContext,
} from '#/hooks/use-customers'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import {
	type AccountExtras,
	useSaveAccountExtras,
} from '#/pages/customers/feature/mocks/account-extras'
import {
	type AccountForm,
	extrasOfForm,
	isAddressEmpty,
	isContactEmpty,
	registrationNumberOf,
} from '#/pages/customers/lib/account-form'
import { todayIso } from '#/pages/customers/lib/format'

export function addressLineOf(street: string, complement: string): string {
	return [street.trim(), complement.trim()].filter(Boolean).join(', ')
}

export function useCreateCustomerAccount(
	organizationId: string,
	organizationSlug: string,
) {
	const navigate = useNavigate()
	const createCustomer = useCreateCustomer(organizationId)
	const createContact = useCreateCustomerContact()
	const createContext = useCreateCustomerContext()
	const saveExtras = useSaveAccountExtras()

	async function create(form: AccountForm) {
		const contact = isContactEmpty(form.contact) ? null : form.contact
		const response = await createCustomer.mutateAsync({
			path: { organization_id: organizationId },
			body: {
				status: form.status === 'client' ? 'CLIENT' : 'PROSPECT',
				pipeline_stage: form.status === 'client' ? 'WON' : form.pipelineStage,
				name: form.name.trim(),
				registration_number: registrationNumberOf(form),
				email: contact?.email.trim() || null,
				phone: contact?.phone.trim() || null,
			},
		})
		const customerId = response.data.id
		const extras = extrasOfForm(form)
		const today = todayIso()
		const patch: Partial<AccountExtras> = {
			...extras,
			convertedAt: form.status === 'client' ? today : null,
		}
		if (contact) {
			const created = await createContact.mutateAsync({
				path: { customer_id: customerId },
				body: {
					first_name: contact.firstName.trim(),
					last_name: contact.lastName.trim(),
					role: contact.role.trim() || null,
					email: contact.email.trim() || null,
					phone: contact.phone.trim() || null,
					is_primary: true,
				},
			})
			const contactId = created.data.id
			patch.billingContactId = contactId
			const mobile = contact.mobile.trim()
			if (mobile) patch.mobiles = { [contactId]: mobile }
		}
		if (!isAddressEmpty(form.address)) {
			const created = await createContext.mutateAsync({
				path: { customer_id: customerId },
				body: {
					label: form.kind === 'particulier' ? 'Domicile' : 'Siège',
					address_line:
						addressLineOf(form.address.street, form.address.complement) || null,
					postal_code: form.address.postalCode.trim() || null,
					city: form.address.city.trim() || null,
					photo_key: null,
				},
			})
			const contextId = created.data.id
			patch.billingContextId = contextId
			patch.addressKinds = { [contextId]: 'billing' }
			patch.addressCountries = {
				[contextId]: form.address.country.trim() || 'France',
			}
		}
		await saveExtras.mutateAsync(customerId, patch)
		void navigate({
			to: buildOrgPath(organizationSlug, '/crm/clients/$customerId'),
			params: { customerId },
		})
	}

	return {
		create,
		isPending:
			createCustomer.isPending ||
			createContact.isPending ||
			createContext.isPending,
		error:
			mutationErrorMessage(createCustomer.error) ??
			mutationErrorMessage(createContact.error) ??
			mutationErrorMessage(createContext.error) ??
			null,
	}
}
