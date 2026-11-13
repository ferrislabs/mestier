import { type ReactNode, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { mutationErrorMessage } from '#/lib/api-error'
import { countOf } from '#/pages/cursus/nav/lib/count-of'
import {
	SCHOOL_TUTORS_PER_PAGE,
	useArchiveSchoolTutor,
	useRestoreSchoolTutor,
	useSchoolTutors,
	useUpdateSchoolTutor,
} from '#/pages/cursus/school-tutors/feature/use-school-tutors'
import {
	type SchoolTutor,
	toJobTitleRequest,
} from '#/pages/cursus/school-tutors/lib/school-tutors'
import { JobTitleDialog } from '#/pages/cursus/school-tutors/ui/job-title-dialog'
import {
	type SchoolTutorRowVM,
	SchoolTutorsListUI,
} from '#/pages/cursus/school-tutors/ui/school-tutors-list-ui'
import { useDebouncedSearch } from '#/pages/cursus/shared/feature/use-debounced-search'
import {
	type ArchiveFilter,
	searchQuery,
} from '#/pages/cursus/shared/lib/list-filters'

export interface SchoolTutorsListFeatureProps {
	tabs: ReactNode
	canManage: boolean
	onCreate: () => void
}

function toRow(tutor: SchoolTutor): SchoolTutorRowVM {
	return {
		id: tutor.id,
		name: tutor.member_name,
		jobTitle: tutor.job_title ?? null,
		openTriadCount: tutor.open_triad_count,
		archived: Boolean(tutor.archived_at),
	}
}

export function SchoolTutorsListFeature({
	tabs,
	canManage,
	onCreate,
}: SchoolTutorsListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const [page, setPage] = useState(1)
	const [archived, setArchived] = useState<ArchiveFilter>('active')
	const [editing, setEditing] = useState<SchoolTutor | null>(null)
	const search = useDebouncedSearch('', () => setPage(1))

	const list = useSchoolTutors(activeOrganizationId, {
		archived,
		...searchQuery(search.query),
		page,
		per_page: SCHOOL_TUTORS_PER_PAGE,
	})
	const update = useUpdateSchoolTutor()
	const archive = useArchiveSchoolTutor()
	const restore = useRestoreSchoolTutor()

	const tutors = list.data?.data ?? []
	const total = countOf(list.data)
	const pageCount = Math.max(
		1,
		Math.ceil((total ?? 0) / SCHOOL_TUTORS_PER_PAGE),
	)
	const path = (id: string) => ({
		organization_id: activeOrganizationId,
		school_tutor_id: id,
	})

	return (
		<>
			<SchoolTutorsListUI
				tabs={tabs}
				rows={tutors.map(toRow)}
				total={total}
				search={search.text}
				archived={archived}
				isLoading={list.isLoading}
				error={mutationErrorMessage(
					list.error ?? archive.error ?? restore.error,
				)}
				canManage={canManage}
				page={page}
				pageCount={pageCount}
				onSearchChange={search.setText}
				onArchivedChange={(next) => {
					setPage(1)
					setArchived(next)
				}}
				onResetFilters={() => {
					search.reset()
					setPage(1)
					setArchived('active')
				}}
				onPageChange={setPage}
				onCreate={onCreate}
				onEditJobTitle={(id) =>
					setEditing(tutors.find((tutor) => tutor.id === id) ?? null)
				}
				onArchive={(id) => archive.mutate({ path: path(id) })}
				onRestore={(id) => restore.mutate({ path: path(id) })}
			/>
			{editing ? (
				<JobTitleDialog
					open
					tutorName={editing.member_name}
					initial={editing.job_title ?? ''}
					isPending={update.isPending}
					error={mutationErrorMessage(update.error)}
					onOpenChange={(next) => {
						if (next) return
						update.reset()
						setEditing(null)
					}}
					onSubmit={(jobTitle) =>
						update.mutate(
							{ path: path(editing.id), body: toJobTitleRequest(jobTitle) },
							{ onSuccess: () => setEditing(null) },
						)
					}
				/>
			) : null}
		</>
	)
}
