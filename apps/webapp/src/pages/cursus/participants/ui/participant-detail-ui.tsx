import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Field } from '#/components/ui/field'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	type ApprenticeValues,
	type IdentityValues,
	participantName,
	STATUS_LABELS,
	toApprenticeValues,
	toIdentityValues,
} from '#/pages/cursus/participants/lib/participants'
import type { HostCompanyOption } from '#/pages/cursus/participants/ui/create-participant-dialog'
import {
	FieldInput,
	FormError,
} from '#/pages/cursus/participants/ui/field-input'

export interface ParticipantDetailErrors {
	identity: string | null
	apprentice: string | null
	tutors: string | null
	status: string | null
}

export interface ParticipantDetailUIProps {
	participant: Schemas.ParticipantDetailResponse | null
	isLoading: boolean
	loadError: string | null
	canManage: boolean
	isBusy: boolean
	hostCompanies: HostCompanyOption[]
	errors: ParticipantDetailErrors
	onSaveIdentity: (values: IdentityValues) => void
	onSaveApprentice: (values: ApprenticeValues) => void
	onAddTutor: (values: { hostCompanyId: string; jobTitle: string }) => void
	onRemoveTutor: (companyTutorId: string) => void
	onInvite: () => void
	onDeactivate: () => void
	onReactivate: () => void
}

export function ParticipantDetailUI({
	participant,
	isLoading,
	loadError,
	canManage,
	isBusy,
	hostCompanies,
	errors,
	onSaveIdentity,
	onSaveApprentice,
	onAddTutor,
	onRemoveTutor,
	onInvite,
	onDeactivate,
	onReactivate,
}: ParticipantDetailUIProps) {
	if (!participant) {
		return (
			<div className="flex flex-col gap-4 p-6">
				{isLoading ? (
					<p className="text-sm text-muted-foreground">Chargement…</p>
				) : null}
				<FormError message={loadError} />
			</div>
		)
	}

	const inviteLabel = participant.invitation_expires_at
		? "Renvoyer l'invitation"
		: 'Inviter'

	return (
		<div className="flex max-w-3xl flex-col gap-6 p-6">
			<div className="flex flex-wrap items-center justify-between gap-3">
				<div className="flex items-center gap-3">
					<h1 className="text-xl font-semibold">
						{participantName(participant)}
					</h1>
					<Badge
						variant={participant.status === 'active' ? 'default' : 'secondary'}
					>
						{STATUS_LABELS[participant.status]}
					</Badge>
				</div>
				{canManage ? (
					<div className="flex gap-2">
						{participant.status === 'pending' ? (
							<Button type="button" disabled={isBusy} onClick={onInvite}>
								{inviteLabel}
							</Button>
						) : null}
						{participant.status === 'inactive' ? (
							<Button
								type="button"
								variant="outline"
								disabled={isBusy}
								onClick={onReactivate}
							>
								Réactiver
							</Button>
						) : (
							<Button
								type="button"
								variant="outline"
								disabled={isBusy}
								onClick={onDeactivate}
							>
								Désactiver
							</Button>
						)}
					</div>
				) : null}
			</div>
			<FormError message={errors.status} />

			<IdentityForm
				key={`identity-${participant.updated_at}`}
				participant={participant}
				canManage={canManage}
				isBusy={isBusy}
				error={errors.identity}
				onSave={onSaveIdentity}
			/>

			{participant.apprentice || canManage ? (
				<ApprenticeForm
					key={`apprentice-${participant.apprentice?.updated_at ?? 'none'}`}
					apprentice={participant.apprentice}
					canManage={canManage}
					isBusy={isBusy}
					error={errors.apprentice}
					onSave={onSaveApprentice}
				/>
			) : null}

			<TutorsSection
				tutors={participant.company_tutors}
				hostCompanies={hostCompanies}
				canManage={canManage}
				isBusy={isBusy}
				error={errors.tutors}
				onAdd={onAddTutor}
				onRemove={onRemoveTutor}
			/>
		</div>
	)
}

function IdentityForm({
	participant,
	canManage,
	isBusy,
	error,
	onSave,
}: {
	participant: Schemas.ParticipantResponse
	canManage: boolean
	isBusy: boolean
	error: string | null
	onSave: (values: IdentityValues) => void
}) {
	const form = useForm({
		defaultValues: toIdentityValues(participant),
		onSubmit: ({ value }) => onSave(value),
	})

	return (
		<form
			className="flex flex-col gap-3 rounded-lg border p-4"
			onSubmit={(event) => {
				event.preventDefault()
				void form.handleSubmit()
			}}
		>
			<h2 className="font-medium">Identité</h2>
			<div className="grid grid-cols-2 gap-3">
				<form.Field name="lastName">
					{(field) => (
						<FieldInput
							id="identity-last-name"
							label="Nom"
							value={field.state.value}
							disabled={!canManage}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="firstName">
					{(field) => (
						<FieldInput
							id="identity-first-name"
							label="Prénom"
							value={field.state.value}
							disabled={!canManage}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="email">
					{(field) => (
						<FieldInput
							id="identity-email"
							label="E-mail"
							type="email"
							value={field.state.value}
							disabled={!canManage}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="phone">
					{(field) => (
						<FieldInput
							id="identity-phone"
							label="Téléphone"
							type="tel"
							value={field.state.value}
							disabled={!canManage}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
			</div>
			<FormError message={error} />
			{canManage ? (
				<div>
					<Button type="submit" disabled={isBusy}>
						Enregistrer l'identité
					</Button>
				</div>
			) : null}
		</form>
	)
}

function ApprenticeForm({
	apprentice,
	canManage,
	isBusy,
	error,
	onSave,
}: {
	apprentice: Schemas.ApprenticeResponse | null | undefined
	canManage: boolean
	isBusy: boolean
	error: string | null
	onSave: (values: ApprenticeValues) => void
}) {
	const form = useForm({
		defaultValues: toApprenticeValues(apprentice),
		onSubmit: ({ value }) => onSave(value),
	})

	return (
		<form
			className="flex flex-col gap-3 rounded-lg border p-4"
			onSubmit={(event) => {
				event.preventDefault()
				void form.handleSubmit()
			}}
		>
			<h2 className="font-medium">Fiche apprenti</h2>
			{apprentice ? null : (
				<p className="text-sm text-muted-foreground">
					Aucune fiche apprenti pour ce participant.
				</p>
			)}
			<div className="grid grid-cols-2 gap-3">
				<form.Field name="birthDate">
					{(field) => (
						<FieldInput
							id="apprentice-birth-date"
							label="Date de naissance"
							type="date"
							value={field.state.value}
							disabled={!canManage}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="ineCode">
					{(field) => (
						<FieldInput
							id="apprentice-ine"
							label="Code INE"
							value={field.state.value}
							disabled={!canManage}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<div className="col-span-2">
					<form.Field name="address">
						{(field) => (
							<FieldInput
								id="apprentice-address"
								label="Adresse"
								value={field.state.value}
								disabled={!canManage}
								onChange={field.handleChange}
							/>
						)}
					</form.Field>
				</div>
			</div>
			<FormError message={error} />
			{canManage ? (
				<div>
					<Button type="submit" disabled={isBusy}>
						{apprentice
							? 'Enregistrer la fiche apprenti'
							: 'Créer la fiche apprenti'}
					</Button>
				</div>
			) : null}
		</form>
	)
}

function TutorsSection({
	tutors,
	hostCompanies,
	canManage,
	isBusy,
	error,
	onAdd,
	onRemove,
}: {
	tutors: Schemas.CompanyTutorResponse[]
	hostCompanies: HostCompanyOption[]
	canManage: boolean
	isBusy: boolean
	error: string | null
	onAdd: (values: { hostCompanyId: string; jobTitle: string }) => void
	onRemove: (companyTutorId: string) => void
}) {
	const [hostCompanyId, setHostCompanyId] = useState('')
	const [jobTitle, setJobTitle] = useState('')

	return (
		<section className="flex flex-col gap-3 rounded-lg border p-4">
			<h2 className="font-medium">Fiches tuteur</h2>
			{tutors.length === 0 ? (
				<p className="text-sm text-muted-foreground">Aucune fiche tuteur.</p>
			) : (
				<ul className="flex flex-col divide-y">
					{tutors.map((tutor) => (
						<li
							key={tutor.id}
							className="flex items-center justify-between gap-3 py-2"
						>
							<div>
								<p className="font-medium">{tutor.host_company_name}</p>
								<p className="text-sm text-muted-foreground">
									{tutor.job_title ?? '—'}
								</p>
							</div>
							{canManage ? (
								<Button
									type="button"
									variant="outline"
									size="sm"
									disabled={isBusy}
									aria-label={`Retirer la fiche ${tutor.host_company_name}`}
									onClick={() => onRemove(tutor.id)}
								>
									Retirer
								</Button>
							) : null}
						</li>
					))}
				</ul>
			)}
			<FormError message={error} />
			{canManage ? (
				<div className="grid grid-cols-[1fr_1fr_auto] items-end gap-3">
					<Field label="Entreprise d'accueil" htmlFor="new-tutor-company">
						<Select value={hostCompanyId} onValueChange={setHostCompanyId}>
							<SelectTrigger id="new-tutor-company">
								<SelectValue placeholder="Choisir…" />
							</SelectTrigger>
							<SelectContent>
								{hostCompanies.map((company) => (
									<SelectItem key={company.id} value={company.id}>
										{company.name}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</Field>
					<FieldInput
						id="new-tutor-job"
						label="Fonction"
						value={jobTitle}
						onChange={setJobTitle}
					/>
					<Button
						type="button"
						disabled={isBusy || hostCompanyId === ''}
						onClick={() => {
							onAdd({ hostCompanyId, jobTitle })
							setHostCompanyId('')
							setJobTitle('')
						}}
					>
						Ajouter une fiche tuteur
					</Button>
				</div>
			) : null}
		</section>
	)
}
