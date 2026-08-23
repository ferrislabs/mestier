import { Link } from '@tanstack/react-router'
import { FileWarning } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'

export interface PendingReportsBadgeProps {
	organizationSlug: string
	count: number
}

/**
 * The pending count the calendar and team views show without opening
 * anything — see the issue's own "visible without opening anything"
 * requirement. Renders nothing at zero: a badge for "no reports" would be
 * one more thing to read on a screen that otherwise has nothing to say
 * about the correction loop.
 */
export function PendingReportsBadge({
	organizationSlug,
	count,
}: PendingReportsBadgeProps) {
	if (count <= 0) return null

	return (
		<Button variant="outline" className="gap-1.5" asChild>
			<Link to={buildOrgPath(organizationSlug, '/planning/reports')}>
				<FileWarning className="size-4 text-amber-600 dark:text-amber-500" />
				{count} écart{count > 1 ? 's' : ''} en attente
			</Link>
		</Button>
	)
}
