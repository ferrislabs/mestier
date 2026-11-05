import { type FormEvent, type ReactNode, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import {
	emptyDraft,
	type FieldKey,
	fieldsOf,
	finalizeDraft,
	finalizeFields,
	groupErrors,
	identityOf,
	managerCandidates,
	validateIdentity,
} from '#/pages/hr/lib/rh/member-draft'
import { fullName, updateMember } from '#/pages/hr/lib/rh/members'
import type {
	Member,
	MemberDraft,
	PostalAddress,
} from '#/pages/hr/lib/rh/types'
import { useRhState, useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'

const NO_MANAGER = '__none'

export type MemberFormDialogProps = {
	open: boolean
	onOpenChange: (open: boolean) => void
	member: Member
	onRename: (names: { firstName: string; lastName: string }) => void
	trigger?: ReactNode
}

export function MemberFormDialog({
	open,
	onOpenChange,
	member,
	onRename,
	trigger,
}: MemberFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			{trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<MemberForm
					member={member}
					onClose={() => onOpenChange(false)}
					onRename={onRename}
				/>
			</DialogContent>
		</Dialog>
	)
}

function Section({
	label,
	hint,
	children,
}: {
	label: string
	hint?: string
	children: ReactNode
}) {
	return (
		<section aria-label={label} className="grid gap-3">
			<div>
				<h3 className="font-medium text-sm">{label}</h3>
				{hint && <p className="text-[12px] text-muted-foreground">{hint}</p>}
			</div>
			<div className="grid gap-3 sm:grid-cols-2">{children}</div>
		</section>
	)
}

function MemberForm({
	member,
	onClose,
	onRename,
}: {
	member: Member
	onClose: () => void
	onRename: (names: { firstName: string; lastName: string }) => void
}) {
	const updateRh = useRhUpdate()
	const state = useRhState()
	const baseId = useId()
	const [draft, setDraft] = useState<MemberDraft>(() => ({
		...emptyDraft(todayIso()),
		...fieldsOf(member),
	}))
	const [touched, setTouched] = useState<ReadonlySet<FieldKey>>(new Set())

	const final = finalizeDraft(draft)
	const errors = validateIdentity(finalizeFields(identityOf(draft)))
	const grouped = groupErrors(errors)
	const otherErrors = grouped.other
	const blocked = errors.length > 0
	const managers = managerCandidates(state, member.id)
	const id = (field: string) => `${baseId}-${field}`

	function touch(...fields: FieldKey[]) {
		setTouched((current) => new Set([...current, ...fields]))
	}

	function change(patch: Partial<MemberDraft>, ...fields: FieldKey[]) {
		setDraft((current) => ({ ...current, ...patch }))
		if (fields.length > 0) touch(...fields)
	}

	function changeAddress(patch: Partial<PostalAddress>, ...fields: FieldKey[]) {
		change({ address: { ...draft.address, ...patch } }, ...fields)
	}

	function shown(field: FieldKey): string | undefined {
		return touched.has(field) ? grouped.byField[field] : undefined
	}

	function fieldProps(field: FieldKey, describedBy?: string) {
		const error = shown(field)
		return {
			'aria-invalid': error ? true : undefined,
			'aria-describedby': error ? `${id(field)}-error` : describedBy,
			onBlur: () => touch(field),
		}
	}

	const emailsError = shown('emails')

	function submit(event: FormEvent) {
		event.preventDefault()
		if (blocked) return
		const fields = finalizeFields(identityOf(final))
		void updateRh((current) => updateMember(current, member.id, fields))
		onRename({ firstName: fields.firstName, lastName: fields.lastName })
		onClose()
	}

	return (
		<form onSubmit={submit} noValidate className="flex min-h-0 flex-1 flex-col">
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>{`Modifier ${fullName(member)}`}</DialogTitle>
				<DialogDescription>
					Mets à jour son identité, ses coordonnées et son rattachement. Ses
					contrats et sa semaine type se gèrent dans les rubriques de la fiche.
				</DialogDescription>
			</DialogHeader>

			<div className="grid min-h-0 flex-1 gap-6 overflow-y-auto border-t px-6 py-5">
				<Section label="Identité">
					<FormField
						id={id('firstName')}
						label="Prénom"
						required
						error={shown('firstName')}
					>
						<Input
							id={id('firstName')}
							value={draft.firstName}
							onChange={(event) =>
								change({ firstName: event.target.value }, 'firstName')
							}
							autoComplete="off"
							{...fieldProps('firstName')}
						/>
					</FormField>
					<FormField
						id={id('lastName')}
						label="Nom"
						required
						error={shown('lastName')}
					>
						<Input
							id={id('lastName')}
							value={draft.lastName}
							onChange={(event) =>
								change({ lastName: event.target.value }, 'lastName')
							}
							autoComplete="off"
							{...fieldProps('lastName')}
						/>
					</FormField>
				</Section>

				<Section
					label="Contact"
					hint="Au moins une adresse e-mail, professionnelle ou personnelle."
				>
					<FormField
						id={id('workEmail')}
						label="E-mail professionnel"
						error={shown('workEmail')}
					>
						<Input
							id={id('workEmail')}
							type="email"
							value={draft.workEmail}
							onChange={(event) =>
								change({ workEmail: event.target.value }, 'workEmail', 'emails')
							}
							autoComplete="off"
							placeholder="prenom.nom@entreprise.fr"
							{...fieldProps(
								'workEmail',
								emailsError ? id('emails-error') : undefined,
							)}
						/>
					</FormField>
					<FormField
						id={id('personalEmail')}
						label="E-mail personnel"
						error={shown('personalEmail')}
					>
						<Input
							id={id('personalEmail')}
							type="email"
							value={draft.personalEmail}
							onChange={(event) =>
								change(
									{ personalEmail: event.target.value },
									'personalEmail',
									'emails',
								)
							}
							autoComplete="off"
							{...fieldProps(
								'personalEmail',
								emailsError ? id('emails-error') : undefined,
							)}
						/>
					</FormField>
					{emailsError && (
						<p
							id={id('emails-error')}
							className="text-[12px] text-state-alert sm:col-span-2"
						>
							{emailsError}
						</p>
					)}
					<FormField id={id('mobile')} label="Mobile" error={shown('mobile')}>
						<Input
							id={id('mobile')}
							type="tel"
							value={draft.mobile}
							onChange={(event) =>
								change({ mobile: event.target.value }, 'mobile')
							}
							autoComplete="off"
							placeholder="06 12 34 56 78"
							{...fieldProps('mobile')}
						/>
					</FormField>
					<FormField
						id={id('landline')}
						label="Téléphone fixe"
						error={shown('landline')}
					>
						<Input
							id={id('landline')}
							type="tel"
							value={draft.landline}
							onChange={(event) =>
								change({ landline: event.target.value }, 'landline')
							}
							autoComplete="off"
							placeholder="04 70 12 34 56"
							{...fieldProps('landline')}
						/>
					</FormField>
				</Section>

				<Section label="Adresse">
					<FormField id={id('street')} label="Rue" className="sm:col-span-2">
						<Input
							id={id('street')}
							value={draft.address.street}
							onChange={(event) =>
								changeAddress({ street: event.target.value })
							}
							autoComplete="off"
						/>
					</FormField>
					<FormField
						id={id('complement')}
						label="Complément"
						className="sm:col-span-2"
					>
						<Input
							id={id('complement')}
							value={draft.address.complement}
							onChange={(event) =>
								changeAddress({ complement: event.target.value })
							}
							autoComplete="off"
							placeholder="Bâtiment, étage, lieu-dit…"
						/>
					</FormField>
					<FormField
						id={id('postalCode')}
						label="Code postal"
						error={shown('postalCode')}
					>
						<Input
							id={id('postalCode')}
							inputMode="numeric"
							value={draft.address.postalCode}
							onChange={(event) =>
								changeAddress({ postalCode: event.target.value }, 'postalCode')
							}
							autoComplete="off"
							{...fieldProps('postalCode')}
						/>
					</FormField>
					<FormField id={id('city')} label="Ville">
						<Input
							id={id('city')}
							value={draft.address.city}
							onChange={(event) => changeAddress({ city: event.target.value })}
							autoComplete="off"
						/>
					</FormField>
					<FormField id={id('country')} label="Pays">
						<Input
							id={id('country')}
							value={draft.address.country}
							onChange={(event) =>
								changeAddress({ country: event.target.value })
							}
							autoComplete="off"
						/>
					</FormField>
				</Section>

				<Section label="Poste et rattachement">
					<FormField id={id('jobTitle')} label="Poste">
						<Input
							id={id('jobTitle')}
							value={draft.jobTitle}
							onChange={(event) => change({ jobTitle: event.target.value })}
							autoComplete="off"
							placeholder="Menuisier, chef de chantier…"
						/>
					</FormField>
					<FormField id={id('manager')} label="Responsable">
						<Select
							value={draft.managerId ?? NO_MANAGER}
							onValueChange={(value) =>
								change({ managerId: value === NO_MANAGER ? null : value })
							}
						>
							<SelectTrigger id={id('manager')} className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value={NO_MANAGER}>Aucun</SelectItem>
								{managers.map((candidate) => (
									<SelectItem key={candidate.id} value={candidate.id}>
										{fullName(candidate)}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</FormField>
					<FormField
						id={id('workPermitEndDate')}
						label="Fin du titre de travail"
					>
						<Input
							id={id('workPermitEndDate')}
							type="date"
							value={draft.workPermitEndDate ?? ''}
							onChange={(event) =>
								change({ workPermitEndDate: event.target.value || null })
							}
						/>
					</FormField>
				</Section>

				{otherErrors.length > 0 && (
					<ul className="grid gap-1 text-[13px] text-state-alert">
						{otherErrors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
			</div>

			<div className="flex items-center justify-between gap-3 border-t px-6 py-3">
				<p className="text-[12px] text-muted-foreground">
					{blocked
						? 'Complète les champs marqués d’une * ou corrige les erreurs pour continuer.'
						: ''}
				</p>
				<div className="flex gap-2">
					<DialogClose asChild>
						<Button type="button" variant="outline">
							Annuler
						</Button>
					</DialogClose>
					<Button type="submit" disabled={blocked}>
						Enregistrer
					</Button>
				</div>
			</div>
		</form>
	)
}
