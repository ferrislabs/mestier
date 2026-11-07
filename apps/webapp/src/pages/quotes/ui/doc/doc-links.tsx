import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'

const LINK =
	'rounded-sm outline-none transition-colors hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring'

function useOrgSlug(): string {
	return useActiveOrganization().activeOrganization.slug
}

interface DocLinkProps {
	children: ReactNode
	className?: string
}

export function AccountLink({
	customerId,
	children,
	className,
}: DocLinkProps & { customerId: string }) {
	return (
		<Link
			to={buildOrgPath(useOrgSlug(), '/crm/clients/$customerId')}
			params={{ customerId }}
			className={cn(LINK, className)}
		>
			{children}
		</Link>
	)
}

export function QuoteLink({
	quoteId,
	children,
	className,
}: DocLinkProps & { quoteId: string }) {
	return (
		<Link
			to={buildOrgPath(useOrgSlug(), '/crm/devis/$quoteId')}
			params={{ quoteId }}
			className={cn(LINK, className)}
		>
			{children}
		</Link>
	)
}

export function InvoiceLink({
	invoiceId,
	children,
	className,
}: DocLinkProps & { invoiceId: string }) {
	return (
		<Link
			to={buildOrgPath(useOrgSlug(), '/crm/factures/$invoiceId')}
			params={{ invoiceId }}
			className={cn(LINK, className)}
		>
			{children}
		</Link>
	)
}

export function CreditNoteLink({
	creditNoteId,
	children,
	className,
}: DocLinkProps & { creditNoteId: string }) {
	return (
		<Link
			to={buildOrgPath(useOrgSlug(), '/crm/avoirs/$creditNoteId')}
			params={{ creditNoteId }}
			className={cn(LINK, className)}
		>
			{children}
		</Link>
	)
}
