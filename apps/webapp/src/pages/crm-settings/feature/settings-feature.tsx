import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useBillingSettings,
	useNumbering,
	useReplaceBillingSettings,
	useSetNextNumber,
} from '#/hooks/use-billing-settings'
import { useUpdateLegalIdentity } from '#/hooks/use-organizations'
import {
	useReplaceStripeSecret,
	useStripeSettings,
} from '#/pages/crm-settings/feature/mocks/settings'
import {
	billingBody,
	billingOf,
	legalIdentityBody,
	sequenceRowsOf,
	seriesOfKind,
	settingsOf,
} from '#/pages/crm-settings/lib/settings-api'
import { cleanCompany } from '#/pages/crm-settings/lib/settings-model'
import type { Company, SequenceKind } from '#/pages/crm-settings/types'
import { SettingsUI } from '#/pages/crm-settings/ui/settings-ui'
import { todayIso } from '#/pages/customers/lib/format'

export function SettingsFeature() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const billing = useBillingSettings(organizationId)
	const numbering = useNumbering(organizationId)
	const stripe = useStripeSettings(activeOrganization.slug)
	const updateLegalIdentity = useUpdateLegalIdentity(organizationId)
	const replaceBilling = useReplaceBillingSettings(organizationId)
	const setNextNumber = useSetNextNumber(organizationId)
	const replaceSecret = useReplaceStripeSecret()
	const [companyError, setCompanyError] = useState<string | null>(null)
	const year = Number(todayIso().slice(0, 4))

	if (!billing.data || !numbering.data)
		return (
			<output className="block p-6 text-muted-foreground text-sm">
				Chargement…
			</output>
		)

	const values = billingOf(billing.data.data)
	const path = { organization_id: organizationId }

	async function saveCompany(company: Company) {
		const cleaned = cleanCompany(company)
		setCompanyError(null)
		try {
			await updateLegalIdentity.mutateAsync({
				path,
				body: legalIdentityBody(activeOrganization, cleaned),
			})
			await replaceBilling.mutateAsync({
				path,
				body: billingBody(values, { iban: cleaned.iban, bic: cleaned.bic }),
			})
		} catch (error) {
			setCompanyError(
				error instanceof Error ? error.message : 'Enregistrement impossible.',
			)
			throw error
		}
	}

	async function saveBilling(patch: Parameters<typeof billingBody>[0]) {
		await replaceBilling.mutateAsync({
			path,
			body: billingBody(patch, { iban: values.iban, bic: values.bic }),
		})
	}

	async function resumeSequences(
		resumes: readonly { kind: SequenceKind; nextNumber: number }[],
	) {
		for (const resume of resumes)
			await setNextNumber.mutateAsync({
				path: { ...path, kind: seriesOfKind(resume.kind) },
				body: { next_number: resume.nextNumber },
			})
	}

	return (
		<SettingsUI
			organizationSlug={activeOrganization.slug}
			settings={settingsOf(activeOrganization, values, stripe)}
			year={year}
			sequenceRows={sequenceRowsOf(numbering.data.data)}
			errors={{
				company: companyError,
				billing: replaceBilling.error?.message ?? null,
				numbering: setNextNumber.error?.message ?? null,
			}}
			onSaveCompany={saveCompany}
			onSaveBilling={saveBilling}
			onResumeSequences={resumeSequences}
			onReplaceSecret={replaceSecret.mutateAsync}
		/>
	)
}
