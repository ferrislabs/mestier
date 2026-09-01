import type { Organization } from '#/hooks/use-organizations'

interface QuoteIssuerBlockProps {
	organization: Organization
}

/**
 * The letterhead: who is issuing the quote. Every field here already exists
 * on the organization's legal identity (#310) — this only lays it out the way
 * a paper quote would, and skips whatever hasn't been filled in rather than
 * printing a blank.
 */
export function QuoteIssuerBlock({ organization }: QuoteIssuerBlockProps) {
	const name = organization.legal_name || organization.name
	const addressLines = [
		organization.address_line1,
		organization.address_line2,
		[organization.address_postal_code, organization.address_city]
			.filter(Boolean)
			.join(' '),
		organization.address_country,
	].filter((line): line is string => Boolean(line?.trim()))

	const legalMentions = [
		organization.legal_form,
		organization.registration_number
			? `SIRET ${organization.registration_number}`
			: null,
		organization.share_capital_cents
			? `Capital de ${(organization.share_capital_cents / 100).toLocaleString('fr-FR')} €`
			: null,
	].filter((mention): mention is string => Boolean(mention))

	const contactLine = [organization.contact_email, organization.contact_phone]
		.filter(Boolean)
		.join(' · ')

	return (
		<div className="flex items-start gap-4">
			<div className="flex size-14 shrink-0 items-center justify-center bg-primary text-lg font-bold text-primary-foreground">
				{organizationInitials(name)}
			</div>
			<div className="min-w-0">
				<p className="text-lg font-bold">{name}</p>
				{addressLines.map((line) => (
					<p key={line} className="text-sm text-muted-foreground">
						{line}
					</p>
				))}
				{legalMentions.length > 0 ? (
					<p className="mt-1 text-xs text-muted-foreground">
						{legalMentions.join(' · ')}
					</p>
				) : null}
				{contactLine ? (
					<p className="text-xs text-muted-foreground">{contactLine}</p>
				) : null}
				{organization.insurance_mention ? (
					<p className="text-xs text-muted-foreground">
						{organization.insurance_mention}
					</p>
				) : null}
			</div>
		</div>
	)
}

function organizationInitials(name: string): string {
	const initials = name
		.split(/\s+/)
		.filter(Boolean)
		.slice(0, 2)
		.map((part) => part[0]?.toUpperCase() ?? '')
		.join('')
	return initials || '?'
}
