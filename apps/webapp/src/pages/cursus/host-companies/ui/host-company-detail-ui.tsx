import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	type HostCompanyValues,
	toHostCompanyValues,
} from '#/pages/cursus/host-companies/lib/host-companies'
import { HostCompanyForm } from '#/pages/cursus/host-companies/ui/host-company-form'
import { formatDate } from '#/pages/cursus/participants/lib/forms'
import { FormError } from '#/pages/cursus/participants/ui/field-input'

export interface HostCompanyTutorVM {
	participantId: string
	name: string
	jobTitle: string | null
}

export interface HostCompanyDetailUIProps {
	company: Schemas.HostCompanyResponse | null
	isLoading: boolean
	loadError: string | null
	canManage: boolean
	isSaving: boolean
	saveError: string | null
	isArchiving: boolean
	archiveError: string | null
	tutors: HostCompanyTutorVM[]
	onSave: (values: HostCompanyValues) => void
	onArchive: () => void
	onOpenParticipant: (participantId: string) => void
}

export function HostCompanyDetailUI({
	company,
	isLoading,
	loadError,
	canManage,
	isSaving,
	saveError,
	isArchiving,
	archiveError,
	tutors,
	onSave,
	onArchive,
	onOpenParticipant,
}: HostCompanyDetailUIProps) {
	if (!company) {
		return (
			<div className="flex flex-col gap-4 p-6">
				{isLoading ? (
					<p className="text-sm text-muted-foreground">Chargement…</p>
				) : null}
				<FormError message={loadError} />
			</div>
		)
	}

	const archived = Boolean(company.archived_at)

	return (
		<div className="flex max-w-3xl flex-col gap-6 p-6">
			<div className="flex flex-wrap items-center justify-between gap-3">
				<div className="flex items-center gap-3">
					<h1 className="text-xl font-semibold">{company.name}</h1>
					{archived ? (
						<Badge variant="secondary">
							Archivée le {formatDate(company.archived_at)}
						</Badge>
					) : null}
				</div>
				{canManage && !archived ? (
					<Button
						type="button"
						variant="outline"
						disabled={isArchiving}
						onClick={onArchive}
					>
						Archiver
					</Button>
				) : null}
			</div>
			<FormError message={archiveError} />

			<section className="rounded-lg border p-4">
				<HostCompanyForm
					key={company.updated_at}
					initialValues={toHostCompanyValues(company)}
					disabled={!canManage || archived}
					submitLabel="Enregistrer"
					isPending={isSaving}
					error={saveError}
					onSubmit={onSave}
				/>
			</section>

			<section className="flex flex-col gap-3 rounded-lg border p-4">
				<h2 className="font-medium">Tuteurs de l'entreprise</h2>
				{tutors.length === 0 ? (
					<p className="text-sm text-muted-foreground">Aucun tuteur.</p>
				) : (
					<ul className="flex flex-col divide-y">
						{tutors.map((tutor) => (
							<li key={tutor.participantId} className="py-2">
								<button
									type="button"
									className="font-medium underline-offset-4 hover:underline"
									onClick={() => onOpenParticipant(tutor.participantId)}
								>
									{tutor.name}
								</button>
								<p className="text-sm text-muted-foreground">
									{tutor.jobTitle ?? '—'}
								</p>
							</li>
						))}
					</ul>
				)}
			</section>
		</div>
	)
}
