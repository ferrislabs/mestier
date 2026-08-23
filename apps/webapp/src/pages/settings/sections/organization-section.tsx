import { useForm } from '@tanstack/react-form'
import {
	AlertTriangle,
	Building2,
	CheckCircle2,
	Loader2,
	Save,
} from 'lucide-react'
import { type FormBinding, TextField } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Organization,
	useUpdateLegalIdentity,
	useUpdateOrganization,
	type VatStatus,
} from '#/hooks/use-organizations'
import {
	LEGAL_IDENTITY_FIELD_LABELS,
	type LegalIdentityFormValues,
	type OrganizationFormValues,
} from '#/pages/settings/types'

export function OrganizationSection() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<div className="flex flex-col gap-6">
			<OrganizationSectionContent
				key={activeOrganization.id}
				organization={activeOrganization}
			/>
			<LegalIdentitySection
				key={`${activeOrganization.id}-legal-identity`}
				organization={activeOrganization}
			/>
		</div>
	)
}

interface OrganizationSectionContentProps {
	organization: Organization
}

function OrganizationSectionContent({
	organization,
}: OrganizationSectionContentProps) {
	const organizationId = organization.id
	const updateOrganization = useUpdateOrganization(organizationId)
	const error = updateOrganization.error

	const organizationForm = useForm({
		defaultValues: {
			name: organization.name,
			slug: organization.slug,
		} satisfies OrganizationFormValues,
		onSubmit: async ({ value }) => {
			await updateOrganization.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: value.name.trim(),
					slug: normalizeSlugForPayload(value.slug),
				},
			})
		},
	})

	return (
		<organizationForm.Subscribe selector={(state) => state.values}>
			{(values) => {
				const form: FormBinding<OrganizationFormValues> = {
					values,
					isPending: updateOrganization.isPending,
					onChange: (patch) => {
						for (const key of Object.keys(
							patch,
						) as (keyof OrganizationFormValues)[]) {
							organizationForm.setFieldValue(key, patch[key] ?? '')
						}
					},
					onSubmit: () => void organizationForm.handleSubmit(),
				}

				const hasChanges =
					form.values.name.trim() !== organization.name ||
					form.values.slug.trim() !== organization.slug

				return (
					<div className="flex flex-col gap-6">
						{error ? (
							<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{error.message}
							</div>
						) : null}
						<SectionCard>
							<SectionHeader
								title="Organisation"
								description="Informations visibles dans l’application et utilisées pour identifier l’espace de travail."
								actions={
									<StatusBadge tone="brand">
										<Building2 className="mr-1 size-3" />
										{organization.slug}
									</StatusBadge>
								}
							/>
							<div className="grid grid-cols-1 gap-4 p-5 lg:grid-cols-[1fr_auto] lg:items-end">
								<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
									<TextField
										label="Nom"
										value={form.values.name}
										onChange={(name) => form.onChange({ name })}
										placeholder="Nom de l’entreprise"
									/>
									<TextField
										label="Identifiant"
										value={form.values.slug}
										onChange={(slug) =>
											form.onChange({ slug: normalizeSlugForDisplay(slug) })
										}
										placeholder="mon-entreprise"
										className="font-mono text-sm"
									/>
								</div>
								<Button
									type="button"
									onClick={form.onSubmit}
									disabled={form.isPending || !hasChanges}
									className="gap-2"
								>
									{form.isPending ? (
										<Loader2 className="size-4 animate-spin" />
									) : (
										<Save className="size-4" />
									)}
									Enregistrer
								</Button>
							</div>
						</SectionCard>
					</div>
				)
			}}
		</organizationForm.Subscribe>
	)
}

// Divergence preserved intentionally (pre-existing, identical to main): display normalization drops the trailing-dash trim that the payload normalization applies — still needs a decision.
function normalizeSlugForDisplay(value: string): string {
	return value
		.toLowerCase()
		.trim()
		.replace(/\s+/g, '-')
		.replace(/[^a-z0-9-]/g, '')
		.replace(/-{2,}/g, '-')
}

function normalizeSlugForPayload(value: string): string {
	return value
		.toLowerCase()
		.trim()
		.replace(/\s+/g, '-')
		.replace(/[^a-z0-9-]/g, '')
		.replace(/-{2,}/g, '-')
		.replace(/^-|-$/g, '')
}

function organizationToLegalIdentityForm(
	organization: Organization,
): LegalIdentityFormValues {
	return {
		legalName: organization.legal_name ?? '',
		legalForm: organization.legal_form ?? '',
		registrationNumber: organization.registration_number ?? '',
		vatChoice: organization.vat_status
			? organization.vat_status.type
			: 'undecided',
		vatNumber:
			organization.vat_status?.type === 'subject'
				? organization.vat_status.vat_number
				: '',
		vatExemptionBasis:
			organization.vat_status?.type === 'not_subject'
				? organization.vat_status.basis
				: '',
		shareCapital:
			organization.share_capital_cents != null
				? (organization.share_capital_cents / 100).toString()
				: '',
		addressLine1: organization.address_line1 ?? '',
		addressLine2: organization.address_line2 ?? '',
		addressPostalCode: organization.address_postal_code ?? '',
		addressCity: organization.address_city ?? '',
		addressCountry: organization.address_country ?? '',
		contactEmail: organization.contact_email ?? '',
		contactPhone: organization.contact_phone ?? '',
		insuranceMention: organization.insurance_mention ?? '',
	}
}

function blankToNull(value: string): string | null {
	const trimmed = value.trim()
	return trimmed ? trimmed : null
}

function legalIdentityFormToVatStatus(
	values: LegalIdentityFormValues,
): VatStatus | null {
	if (values.vatChoice === 'subject') {
		return { type: 'subject', vat_number: values.vatNumber.trim() }
	}
	if (values.vatChoice === 'not_subject') {
		return { type: 'not_subject', basis: values.vatExemptionBasis.trim() }
	}
	return null
}

interface LegalIdentitySectionProps {
	organization: Organization
}

/**
 * Identity, address, VAT and insurance — everything a document needs before
 * it can be legally issued (#310). Extends the organization section rather
 * than living apart: this is the organization, not a separate concern.
 */
function LegalIdentitySection({ organization }: LegalIdentitySectionProps) {
	const organizationId = organization.id
	const updateLegalIdentity = useUpdateLegalIdentity(organizationId)
	const error = updateLegalIdentity.error

	const form = useForm({
		defaultValues: organizationToLegalIdentityForm(organization),
		onSubmit: async ({ value }) => {
			await updateLegalIdentity.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					legal_name: blankToNull(value.legalName),
					legal_form: blankToNull(value.legalForm),
					registration_number: blankToNull(value.registrationNumber),
					vat_status: legalIdentityFormToVatStatus(value),
					share_capital_cents:
						value.shareCapital.trim() === ''
							? null
							: Math.round(Number(value.shareCapital.replace(',', '.')) * 100),
					address_line1: blankToNull(value.addressLine1),
					address_line2: blankToNull(value.addressLine2),
					address_postal_code: blankToNull(value.addressPostalCode),
					address_city: blankToNull(value.addressCity),
					address_country: blankToNull(value.addressCountry),
					contact_email: blankToNull(value.contactEmail),
					contact_phone: blankToNull(value.contactPhone),
					insurance_mention: blankToNull(value.insuranceMention),
				},
			})
		},
	})

	const missingFields = organization.missing_legal_identity_fields

	return (
		<form.Subscribe selector={(state) => state.values}>
			{(values) => {
				const binding: FormBinding<LegalIdentityFormValues> = {
					values,
					isPending: updateLegalIdentity.isPending,
					onChange: (patch) => {
						for (const key of Object.keys(
							patch,
						) as (keyof LegalIdentityFormValues)[]) {
							const next = patch[key]
							if (next !== undefined) form.setFieldValue(key, next)
						}
					},
					onSubmit: () => void form.handleSubmit(),
				}

				const canSubmit = binding.values.vatChoice !== 'undecided'

				return (
					<SectionCard>
						<SectionHeader
							title="Identité légale"
							description="Les mentions obligatoires pour qu'un devis ou une facture soit valide."
							actions={
								missingFields.length === 0 ? (
									<StatusBadge tone="brand">
										<CheckCircle2 className="mr-1 size-3" />
										Complète
									</StatusBadge>
								) : null
							}
						/>

						{error ? (
							<div className="mx-5 mt-5 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{error.message}
							</div>
						) : null}

						{missingFields.length > 0 ? (
							<div className="mx-5 mt-5 flex items-start gap-3 rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-800 dark:text-amber-300">
								<AlertTriangle className="mt-0.5 size-4 shrink-0" />
								<p>
									Tant que cette identité est incomplète, aucun devis ne peut
									être envoyé ou exporté en PDF. Il manque :{' '}
									{missingFields.map((field, index) => (
										<span key={field}>
											{index > 0 ? ', ' : ''}
											<a
												href={`#${field}`}
												className="font-medium underline underline-offset-2"
											>
												{LEGAL_IDENTITY_FIELD_LABELS[field] ?? field}
											</a>
										</span>
									))}
									.
								</p>
							</div>
						) : null}

						<div className="space-y-6 p-5">
							<FieldGroup title="Identité">
								<TextField
									label="Raison sociale"
									id="legal_name"
									value={binding.values.legalName}
									onChange={(legalName) => binding.onChange({ legalName })}
									placeholder="Acme SARL"
								/>
								<TextField
									label="Forme juridique"
									id="legal_form"
									value={binding.values.legalForm}
									onChange={(legalForm) => binding.onChange({ legalForm })}
									placeholder="SARL, EI, auto-entrepreneur…"
								/>
								<TextField
									label="SIRET"
									id="registration_number"
									value={binding.values.registrationNumber}
									onChange={(registrationNumber) =>
										binding.onChange({ registrationNumber })
									}
									placeholder="123 456 789 00012"
								/>
								<TextField
									label="Capital social (€)"
									value={binding.values.shareCapital}
									onChange={(shareCapital) =>
										binding.onChange({ shareCapital })
									}
									placeholder="Optionnel"
									inputMode="decimal"
								/>
							</FieldGroup>

							<FieldGroup title="Adresse">
								<TextField
									label="Adresse"
									id="address_line1"
									value={binding.values.addressLine1}
									onChange={(addressLine1) =>
										binding.onChange({ addressLine1 })
									}
									placeholder="12 rue des Artisans"
								/>
								<TextField
									label="Complément"
									value={binding.values.addressLine2}
									onChange={(addressLine2) =>
										binding.onChange({ addressLine2 })
									}
									placeholder="Optionnel"
								/>
								<TextField
									label="Code postal"
									id="address_postal_code"
									value={binding.values.addressPostalCode}
									onChange={(addressPostalCode) =>
										binding.onChange({ addressPostalCode })
									}
									placeholder="75001"
								/>
								<TextField
									label="Ville"
									id="address_city"
									value={binding.values.addressCity}
									onChange={(addressCity) => binding.onChange({ addressCity })}
									placeholder="Paris"
								/>
								<TextField
									label="Pays"
									id="address_country"
									value={binding.values.addressCountry}
									onChange={(addressCountry) =>
										binding.onChange({ addressCountry })
									}
									placeholder="FR"
								/>
							</FieldGroup>

							<FieldGroup title="TVA">
								<div className="flex flex-col gap-2">
									<Label htmlFor="vat_status">Statut</Label>
									<Select
										value={binding.values.vatChoice}
										onValueChange={(vatChoice) =>
											binding.onChange({
												vatChoice:
													vatChoice as LegalIdentityFormValues['vatChoice'],
											})
										}
									>
										<SelectTrigger id="vat_status" className="w-full">
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="undecided">
												À déterminer — aucun document ne peut être émis
											</SelectItem>
											<SelectItem value="subject">
												Assujetti à la TVA
											</SelectItem>
											<SelectItem value="not_subject">
												Non assujetti (franchise en base…)
											</SelectItem>
										</SelectContent>
									</Select>
								</div>
								{binding.values.vatChoice === 'subject' ? (
									<TextField
										label="Numéro de TVA intracommunautaire"
										value={binding.values.vatNumber}
										onChange={(vatNumber) => binding.onChange({ vatNumber })}
										placeholder="FR12345678901"
									/>
								) : null}
								{binding.values.vatChoice === 'not_subject' ? (
									<TextField
										label="Base légale de l'exonération"
										value={binding.values.vatExemptionBasis}
										onChange={(vatExemptionBasis) =>
											binding.onChange({ vatExemptionBasis })
										}
										placeholder="Article 293 B du CGI"
									/>
								) : null}
							</FieldGroup>

							<FieldGroup title="Assurance et contact">
								<TextField
									label="Assurance professionnelle"
									id="insurance_mention"
									value={binding.values.insuranceMention}
									onChange={(insuranceMention) =>
										binding.onChange({ insuranceMention })
									}
									placeholder="RC Pro n°123456 - Assureur"
								/>
								<TextField
									label="Email de contact"
									value={binding.values.contactEmail}
									onChange={(contactEmail) =>
										binding.onChange({ contactEmail })
									}
									placeholder="Optionnel"
								/>
								<TextField
									label="Téléphone"
									value={binding.values.contactPhone}
									onChange={(contactPhone) =>
										binding.onChange({ contactPhone })
									}
									placeholder="Optionnel"
								/>
							</FieldGroup>

							<div className="flex items-center justify-end gap-4 border-t pt-5">
								<Button
									type="button"
									onClick={binding.onSubmit}
									disabled={binding.isPending || !canSubmit}
									className="gap-2"
								>
									{binding.isPending ? (
										<Loader2 className="size-4 animate-spin" />
									) : (
										<Save className="size-4" />
									)}
									Enregistrer
								</Button>
							</div>
						</div>
					</SectionCard>
				)
			}}
		</form.Subscribe>
	)
}

function FieldGroup({
	title,
	children,
}: {
	title: string
	children: React.ReactNode
}) {
	return (
		<div className="space-y-3">
			<p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
				{title}
			</p>
			<div className="grid grid-cols-1 gap-4 md:grid-cols-2">{children}</div>
		</div>
	)
}
