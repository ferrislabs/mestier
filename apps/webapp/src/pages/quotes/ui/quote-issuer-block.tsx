import type { Organization } from '#/hooks/use-organizations'

interface QuoteIssuerProps {
	organization: Organization
}

/**
 * The initials mark, alone — sits at the top of the document next to the
 * quote number, the way a letterhead logo would. Split from the details
 * block below because a real letterhead only shows the mark once, not next
 * to every line of the address.
 */
export function QuoteIssuerMark({ organization }: QuoteIssuerProps) {
	const name = organization.legal_name || organization.name

	return (
		<div className="flex size-11 shrink-0 items-center justify-center bg-primary text-base font-bold text-primary-foreground">
			{organizationInitials(name)}
		</div>
	)
}

/**
 * Who is issuing the quote, written out the way a letterhead would: name,
 * address, legal mentions, contact — never edited here. Every field already
 * exists on the organization's legal identity (#310); this only lays it out,
 * and skips whatever hasn't been filled in rather than printing a blank.
 */
export function QuoteIssuerDetails({ organization }: QuoteIssuerProps) {
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
		<div className="min-w-0">
			<p className="font-semibold">{name}</p>
			{addressLines.map((line) => (
				<p key={line} className="text-sm text-muted-foreground">
					{line}
				</p>
			))}
			{contactLine ? (
				<p className="text-sm text-muted-foreground">{contactLine}</p>
			) : null}
			{legalMentions.length > 0 ? (
				<p className="mt-1 text-xs text-muted-foreground">
					{legalMentions.join(' · ')}
				</p>
			) : null}
			{organization.insurance_mention ? (
				<p className="text-xs text-muted-foreground">
					{organization.insurance_mention}
				</p>
			) : null}
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
