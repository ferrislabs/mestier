import { PageHeader, PageShell } from '#/components/ui/surface'
import type {
	BillingPatch,
	SequenceRow,
} from '#/pages/crm-settings/lib/settings-model'
import type {
	Company,
	SequenceKind,
	Settings,
} from '#/pages/crm-settings/types'
import { BillingSection } from '#/pages/crm-settings/ui/billing-section'
import { CompanySection } from '#/pages/crm-settings/ui/company-section'
import { NumberingSection } from '#/pages/crm-settings/ui/numbering-section'
import { SettingsNav } from '#/pages/crm-settings/ui/settings-nav'
import { StripeSection } from '#/pages/crm-settings/ui/stripe-section'

export type SettingsErrors = {
	company: string | null
	billing: string | null
	numbering: string | null
}

export function SettingsUI({
	organizationSlug,
	settings,
	year,
	sequenceRows,
	errors,
	onSaveCompany,
	onSaveBilling,
	onResumeSequences,
	onReplaceSecret,
}: {
	organizationSlug: string
	settings: Settings
	year: number
	sequenceRows: readonly SequenceRow[]
	errors: SettingsErrors
	onSaveCompany: (company: Company) => Promise<void>
	onSaveBilling: (patch: BillingPatch) => Promise<void>
	onResumeSequences: (
		resumes: readonly { kind: SequenceKind; nextNumber: number }[],
	) => Promise<void>
	onReplaceSecret: () => Promise<void>
}) {
	const invoicePrefix =
		sequenceRows.find((row) => row.kind === 'invoice')?.prefix ?? 'FAC'
	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6 flex-row flex-wrap items-end justify-between gap-4"
				title="Réglages"
				description="Les mentions de ton entreprise, la facturation, la numérotation et le webhook Stripe."
			/>
			<div className="grid gap-6 lg:grid-cols-[13rem_minmax(0,1fr)] lg:gap-10">
				<SettingsNav />
				<div className="grid min-w-0 content-start gap-6">
					<CompanySection
						stored={settings.company}
						error={errors.company}
						onSave={onSaveCompany}
					/>
					<BillingSection
						settings={settings}
						year={year}
						invoicePrefix={invoicePrefix}
						error={errors.billing}
						onSave={onSaveBilling}
					/>
					<NumberingSection
						rows={sequenceRows}
						padding={settings.numberPadding}
						error={errors.numbering}
						onSave={onResumeSequences}
					/>
					<StripeSection
						organizationSlug={organizationSlug}
						settings={settings}
						onReplaceSecret={onReplaceSecret}
					/>
				</div>
			</div>
		</PageShell>
	)
}
