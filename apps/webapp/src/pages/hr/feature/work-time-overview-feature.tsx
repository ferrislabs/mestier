import { TZDate } from '@date-fns/tz'
import { format } from 'date-fns'
import type { Absence } from '#/hooks/use-absences'
import { useAbsences } from '#/hooks/use-absences'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { isForbiddenError } from '#/lib/api-error'
import { browserTimeZone } from '#/pages/hr/types'
import type {
	WorkTimeOverviewNextAbsence,
	WorkTimeOverviewRow,
} from '#/pages/hr/ui/work-time-overview-ui'
import { WorkTimeOverviewUI } from '#/pages/hr/ui/work-time-overview-ui'

export function WorkTimeOverviewFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkTimeOverviewScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

interface WorkTimeOverviewScreenProps {
	organizationId: string
	organizationSlug: string
}

function WorkTimeOverviewScreen({
	organizationId,
	organizationSlug,
}: WorkTimeOverviewScreenProps) {
	const catalog = useReferenceCatalog(organizationId, {
		equipment: false,
		serviceRates: false,
		products: false,
	})
	const absences = useAbsences(organizationId)

	const isLoading =
		catalog.members.isLoading ||
		catalog.employeeProfiles.isLoading ||
		absences.isLoading

	// See team-list-feature.tsx: `employee-profiles` is gated on a stricter
	// permission than `members`, so a 403 here is an expected access
	// boundary, not a page failure. See #371.
	const hrDataRestricted = isForbiddenError(catalog.employeeProfiles.error)

	const error =
		catalog.members.error ??
		(hrDataRestricted ? null : catalog.employeeProfiles.error) ??
		absences.error

	const members = catalog.members.data?.data ?? []
	const profileByMember = new Map(
		(catalog.employeeProfiles.data?.data ?? []).map((profile) => [
			profile.member_id,
			profile,
		]),
	)
	const allAbsences = absences.data?.data ?? []

	const timeZone = browserTimeZone()
	const now = Date.now()

	const rows: WorkTimeOverviewRow[] = members
		.map((member) => {
			const profile = profileByMember.get(member.id)
			const memberAbsences = allAbsences.filter(
				(absence) => absence.member_id === member.id,
			)
			return {
				memberId: member.id,
				displayName: member.display_name,
				weeklyContractMinutes: profile?.weekly_contract_minutes ?? null,
				nextAbsence: nextUpcomingAbsence(memberAbsences, now, timeZone),
			}
		})
		.sort((a, b) => a.displayName.localeCompare(b.displayName))

	return (
		<WorkTimeOverviewUI
			organizationSlug={organizationSlug}
			isLoading={isLoading}
			error={error?.message ?? null}
			hrDataRestricted={hrDataRestricted}
			rows={rows}
		/>
	)
}

/**
 * The earliest absence starting now or later for one member, or `null` when
 * none is scheduled. `starts_at` is an ISO instant; the returned date is
 * resolved to a local calendar date in `timeZone` (mirrors
 * `absenceToDraft`'s own resolution in `pages/hr/lib/absences.ts`) so the UI
 * layer can format it with `formatDateFr` like every other date on this
 * screen.
 */
function nextUpcomingAbsence(
	memberAbsences: Absence[],
	now: number,
	timeZone: string,
): WorkTimeOverviewNextAbsence | null {
	const upcoming = memberAbsences
		.filter((absence) => new Date(absence.starts_at).getTime() >= now)
		.sort((a, b) => a.starts_at.localeCompare(b.starts_at))

	const earliest = upcoming[0]
	if (!earliest) return null

	return {
		date: format(new TZDate(earliest.starts_at, timeZone), 'yyyy-MM-dd'),
		kind: earliest.kind,
	}
}
