import type { Schemas } from '#/api/api.client'
import { RichTextView } from '#/components/rich-text'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
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

function HostCompanySummary({
	company,
}: {
	company: Schemas.HostCompanyResponse
}) {
	const rows: [string, string | number | null | undefined][] = [
		['Groupe', company.group_name],
		['Numéro de SIRET', company.siret],
		['Code NAF', company.naf_code],
		['Forme juridique', company.legal_form],
		['Capital', company.capital],
		['Nom du directeur', company.director_name],
		['Téléphone', company.phone],
		['Email', company.email],
		['URL site internet', company.website],
		["Nombre d'employés dans l'entreprise", company.employee_count],
		["Nombre d'employés dans le groupe", company.group_employee_count],
		["Domaines d'activité", company.activity_domains],
	]
	return (
		<div className="flex flex-col gap-4">
			<dl className="grid grid-cols-2 gap-3 text-sm">
				{rows.map(([label, value]) => (
					<div key={label}>
						<dt className="text-muted-foreground">{label}</dt>
						<dd>{value ?? '—'}</dd>
					</div>
				))}
			</dl>
			<div className="flex flex-col gap-1">
				<h2 className="text-sm font-medium">Missions et objectifs</h2>
				{company.missions ? (
					<RichTextView markdown={company.missions} />
				) : (
					<p className="text-sm text-muted-foreground">—</p>
				)}
			</div>
			<div className="flex flex-col gap-2">
				<h2 className="text-sm font-medium">Adresses</h2>
				{company.addresses.length === 0 ? (
					<p className="text-sm text-muted-foreground">Aucune adresse.</p>
				) : (
					<ul className="flex flex-col gap-2">
						{company.addresses.map((address) => (
							<li
								key={`${address.line1}-${address.postal_code}-${address.city}`}
								className="flex flex-col rounded-lg border p-3 text-sm"
							>
								<span className="flex items-center gap-2 font-medium">
									{address.label ?? 'Adresse'}
									{address.is_main ? <Badge>Principale</Badge> : null}
								</span>
								<span>{address.line1}</span>
								{address.line2 ? <span>{address.line2}</span> : null}
								<span>
									{[address.postal_code, address.city]
										.filter(Boolean)
										.join(' ')}
								</span>
							</li>
						))}
					</ul>
				)}
			</div>
		</div>
	)
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
			<PageShell>
				{isLoading ? (
					<p className="text-sm text-muted-foreground">Chargement…</p>
				) : null}
				<FormError message={loadError} />
			</PageShell>
		)
	}

	const archived = Boolean(company.archived_at)

	return (
		<PageShell>
			<PageHeader
				title={company.name}
				actions={
					canManage && !archived ? (
						<Button
							type="button"
							variant="outline"
							disabled={isArchiving}
							onClick={onArchive}
						>
							Archiver
						</Button>
					) : null
				}
			>
				{archived ? (
					<Badge variant="secondary" className="mt-2">
						Archivée le {formatDate(company.archived_at)}
					</Badge>
				) : null}
			</PageHeader>
			<FormError message={archiveError} />

			<section className="rounded-lg border p-4">
				{canManage && !archived ? (
					<HostCompanyForm
						key={company.updated_at}
						initialValues={toHostCompanyValues(company)}
						submitLabel="Enregistrer"
						isPending={isSaving}
						error={saveError}
						onSubmit={onSave}
					/>
				) : (
					<HostCompanySummary company={company} />
				)}
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
		</PageShell>
	)
}
