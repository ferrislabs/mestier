import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Field } from '#/components/ui/field'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	type CreateParticipantValues,
	EMPTY_CREATE_PARTICIPANT,
	KIND_LABELS,
	type ParticipantKind,
	validateCreateParticipant,
} from '#/pages/cursus/participants/lib/participants'
import {
	FieldInput,
	FormError,
} from '#/pages/cursus/participants/ui/field-input'

export interface HostCompanyOption {
	id: string
	name: string
}

export interface CreateParticipantDialogProps {
	open: boolean
	hostCompanies: HostCompanyOption[]
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: CreateParticipantValues) => void
}

export function CreateParticipantDialog({
	open,
	hostCompanies,
	isPending,
	error,
	onOpenChange,
	onSubmit,
}: CreateParticipantDialogProps) {
	const [formError, setFormError] = useState<string | null>(null)
	const form = useForm({
		defaultValues: EMPTY_CREATE_PARTICIPANT,
		onSubmit: ({ value }) => {
			const problem = validateCreateParticipant(value)
			setFormError(problem)
			if (problem === null) onSubmit(value)
		},
	})

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-lg">
				<DialogHeader>
					<DialogTitle>Nouveau participant</DialogTitle>
					<DialogDescription>
						Créez un apprenti ou un tuteur d'entreprise. Vous pourrez ensuite
						l'inviter à rejoindre son espace.
					</DialogDescription>
				</DialogHeader>
				<form
					className="flex flex-col gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						void form.handleSubmit()
					}}
				>
					<div className="grid grid-cols-2 gap-3">
						<form.Field name="lastName">
							{(field) => (
								<FieldInput
									id="participant-last-name"
									label="Nom"
									value={field.state.value}
									onChange={field.handleChange}
								/>
							)}
						</form.Field>
						<form.Field name="firstName">
							{(field) => (
								<FieldInput
									id="participant-first-name"
									label="Prénom"
									value={field.state.value}
									onChange={field.handleChange}
								/>
							)}
						</form.Field>
						<form.Field name="email">
							{(field) => (
								<FieldInput
									id="participant-email"
									label="E-mail"
									type="email"
									value={field.state.value}
									onChange={field.handleChange}
								/>
							)}
						</form.Field>
						<form.Field name="phone">
							{(field) => (
								<FieldInput
									id="participant-phone"
									label="Téléphone"
									type="tel"
									value={field.state.value}
									onChange={field.handleChange}
								/>
							)}
						</form.Field>
					</div>

					<form.Field name="kind">
						{(field) => (
							<Field label="Type de participant" htmlFor="participant-kind">
								<Select
									value={field.state.value}
									onValueChange={(value) =>
										field.handleChange(value as ParticipantKind)
									}
								>
									<SelectTrigger id="participant-kind">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{(Object.keys(KIND_LABELS) as ParticipantKind[]).map(
											(kind) => (
												<SelectItem key={kind} value={kind}>
													{KIND_LABELS[kind]}
												</SelectItem>
											),
										)}
									</SelectContent>
								</Select>
							</Field>
						)}
					</form.Field>

					<form.Subscribe selector={(state) => state.values.kind}>
						{(kind) =>
							kind === 'apprentice' ? (
								<div className="grid grid-cols-2 gap-3">
									<form.Field name="birthDate">
										{(field) => (
											<FieldInput
												id="participant-birth-date"
												label="Date de naissance"
												type="date"
												value={field.state.value}
												onChange={field.handleChange}
											/>
										)}
									</form.Field>
									<form.Field name="ineCode">
										{(field) => (
											<FieldInput
												id="participant-ine"
												label="Code INE"
												value={field.state.value}
												onChange={field.handleChange}
											/>
										)}
									</form.Field>
									<div className="col-span-2">
										<form.Field name="address">
											{(field) => (
												<FieldInput
													id="participant-address"
													label="Adresse"
													value={field.state.value}
													onChange={field.handleChange}
												/>
											)}
										</form.Field>
									</div>
								</div>
							) : (
								<div className="grid grid-cols-2 gap-3">
									<form.Field name="hostCompanyId">
										{(field) => (
											<Field
												label="Entreprise d'accueil"
												htmlFor="participant-host-company"
											>
												<Select
													value={field.state.value}
													onValueChange={field.handleChange}
												>
													<SelectTrigger id="participant-host-company">
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
										)}
									</form.Field>
									<form.Field name="jobTitle">
										{(field) => (
											<FieldInput
												id="participant-job-title"
												label="Fonction"
												value={field.state.value}
												onChange={field.handleChange}
											/>
										)}
									</form.Field>
								</div>
							)
						}
					</form.Subscribe>

					<FormError message={formError ?? error} />

					<DialogFooter>
						<Button
							type="button"
							variant="ghost"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={isPending}>
							Créer le participant
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
